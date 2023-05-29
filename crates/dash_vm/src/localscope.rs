use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::gc::handle::Handle;

use super::value::object::Object;
use super::value::Value;
use super::Vm;

use std::ptr::NonNull;

use crate::gc::trace::Trace;

#[derive(Debug)]
pub struct LocalScopeList {
    head: Option<NonNull<ScopeData>>,
}

pub fn scope(vm: &mut Vm) -> LocalScope<'_> {
    match vm.scopes.head {
        Some(ptr) => {
            // We have an available scope we can use.
            let data = unsafe { ptr.as_ref() };
            let next = data.next;
            vm.scopes.head = next;
            LocalScope {
                _p: PhantomData,
                scope_data: ptr,
                vm,
            }
        }
        None => {
            // No scope available.
            let scope_data = ScopeData::new(None);
            LocalScope {
                _p: PhantomData,
                scope_data,
                vm,
            }
        }
    }
}

impl Drop for LocalScopeList {
    fn drop(&mut self) {
        let Self { mut head } = self;

        while let Some(ptr) = head {
            let data = unsafe { ptr.as_ref() };
            head = data.next;
            // SAFETY: ptr was created using Box::into_raw
            unsafe { Box::from_raw(ptr.as_ptr()) };
        }
    }
}

unsafe impl Trace for LocalScopeList {
    fn trace(&self) {
        let Self { mut head } = self;

        while let Some(ptr) = head {
            let data = unsafe { ptr.as_ref() };
            data.refs.trace();
            head = data.next;
        }
    }
}

impl LocalScopeList {
    pub fn new() -> Self {
        let node1 = ScopeData::new(None);
        let node2 = ScopeData::new(Some(node1));
        let node3 = ScopeData::new(Some(node2));
        let node4 = ScopeData::new(Some(node3));
        Self { head: Some(node4) }
    }
}

impl Default for LocalScopeList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct ScopeData {
    refs: Vec<Handle<dyn Object>>,
    next: Option<NonNull<ScopeData>>,
}

impl ScopeData {
    pub fn new(next: Option<NonNull<Self>>) -> NonNull<Self> {
        NonNull::new(Box::into_raw(Box::new(Self {
            refs: Vec::with_capacity(4),
            next,
        })))
        .unwrap()
    }
}

/// A handle to a [`LocalScope`], managed by [`LocalScopeList`].
#[derive(Debug)]
pub struct LocalScope<'vm> {
    vm: *mut Vm,
    scope_data: NonNull<ScopeData>,
    _p: PhantomData<&'vm mut Vm>,
}

impl<'vm> LocalScope<'vm> {
    fn scope_data_mut(&mut self) -> &mut ScopeData {
        unsafe { self.scope_data.as_mut() }
    }

    pub fn add_ref(&mut self, obj: Handle<dyn Object>) {
        self.scope_data_mut().refs.push(obj);
    }

    pub fn add_value(&mut self, value: Value) {
        match value {
            Value::Object(o) => self.add_ref(o),
            Value::External(o) => {
                // Two things to add: the inner object, and the external itself
                self.add_ref(o.inner.clone());
                self.add_ref(o.into_dyn());
            }
            _ => {}
        }
    }

    pub fn add_many(&mut self, mut v: Vec<Handle<dyn Object>>) {
        self.scope_data_mut().refs.append(&mut v);
    }

    pub fn register<O: Object + 'static>(&mut self, obj: O) -> Handle<dyn Object> {
        let handle = self.deref_mut().register(obj);
        self.add_ref(handle.clone());
        handle
    }
}

impl<'a> Deref for LocalScope<'a> {
    type Target = Vm;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.vm }
    }
}

impl<'a> DerefMut for LocalScope<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.vm }
    }
}

impl<'vm> Drop for LocalScope<'vm> {
    fn drop(&mut self) {
        let head = self.scopes.head;
        let data = self.scope_data_mut();
        data.refs.clear();
        data.next = head;
        self.scopes.head = Some(self.scope_data);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::Vm;

    #[test]
    fn it_works() {
        let mut vm = Vm::new(Default::default());
        let mut scope = vm.scope();
        for _ in 0..20 {
            scope.register(Rc::from("test") as Rc<str>);
        }
    }

    #[test]
    fn multiple_scopes() {
        let mut vm = Vm::new(Default::default());
        let mut scope = vm.scope();
        let mut scope1 = scope.scope();
        let mut scope2 = scope1.scope();
        let mut scope3 = scope2.scope();
        let mut scope4 = scope3.scope();
        let mut scope5 = scope4.scope();
        scope5.register(Rc::from("bar") as Rc<str>);
        let mut scope6 = scope5.scope();
        let mut scope7 = scope6.scope();
        let mut scope8 = scope7.scope();
        scope8.register(Rc::from("foo") as Rc<str>);
        let mut scope9 = scope8.scope();
        scope9.register(Rc::from("test") as Rc<str>);
    }
}
