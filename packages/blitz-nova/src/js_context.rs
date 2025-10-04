
use std::{
    borrow::BorrowMut,
};

use nova_vm::{
    ecmascript::{
        execution::{
            agent::{Agent, GcAgent, Options, RealmRoot},
        },
        scripts_and_modules::{
            script::{parse_script, script_evaluation},
        },
        types::{
            InternalMethods, IntoObject, IntoValue, Object, OrdinaryObject, PropertyDescriptor, PropertyKey, String as JsString,
        },
    },
    engine::{
        context::{Bindable, GcScope},
        rootable::Scopable,
    },
};

use crate::{extension::recommended_extensions, host_hooks::HostHandler};
use crate::host_hooks::BlitzHostHooks;
use crate::runtime::RUNTIME_JS;

#[derive(Debug)]
pub enum JSContextError {
    
}

pub struct JSContext {
    agent: GcAgent,
    realm_root: RealmRoot,
}

impl JSContext {
    pub fn new(handler: Box<dyn HostHandler>) -> Self {
        let host_hooks = BlitzHostHooks::new(handler);
        let host_hooks = &*Box::leak(Box::new(host_hooks));
        let mut agent = GcAgent::new(Options::default(), host_hooks);
        
        let create_global_object: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let create_global_this_value: Option<
            for<'a> fn(&mut Agent, GcScope<'a, '_>) -> Object<'a>,
        > = None;
        let realm_root = agent.create_realm(
            create_global_object,
            create_global_this_value,
            Some(
                |agent: &mut Agent, global_object: Object<'_>, mut gc: GcScope<'_, '_>| {
                    let blitz_obj = OrdinaryObject::create_empty_object(agent, gc.nogc())
                        .scope(agent, gc.nogc());
                    let property_key =
                        PropertyKey::from_static_str(agent, "__blitz__", gc.nogc());
                    global_object
                        .internal_define_own_property(
                            agent,
                            property_key.unbind(),
                            PropertyDescriptor {
                                value: Some(blitz_obj.get(agent).into_value()),
                                writable: Some(true),
                                enumerable: Some(false),
                                configurable: Some(true),
                                ..Default::default()
                            },
                            gc.reborrow(),
                        )
                        .unwrap();

                    for extension in &mut recommended_extensions() {
                        extension.load(
                            agent,
                            global_object,
                            blitz_obj.get(agent).into_object(),
                            gc.borrow_mut(),
                        );
                    }
                },
            ),
        );
        
        agent.run_in_realm(&realm_root, |agent, mut gc| {
            let realm = agent.current_realm(gc.nogc());
            let source_text = JsString::from_string(agent, RUNTIME_JS.to_string(), gc.nogc());
            let script = parse_script(agent, source_text, realm, false, None, gc.nogc()).unwrap();
            let result = script_evaluation(agent, script.unbind(), gc.reborrow()).unwrap();
            //println!("JS code: `{RUNTIME_JS}`");
            //println!("Result: {:?}", result); // Should be Value::Number(42.0)
        });
        
        Self {
            agent,
            realm_root,
        }
    }
    
    pub fn run(&mut self, js_code: &str) -> Result<(), JSContextError> {
        self.agent.run_in_realm(&self.realm_root, |agent, mut gc| {
            let realm = agent.current_realm(gc.nogc());
            let source_text = JsString::from_string(agent, js_code.to_string(), gc.nogc());
            let script = parse_script(agent, source_text, realm, true, None, gc.nogc()).unwrap();
            let result = script_evaluation(agent, script.unbind(), gc.reborrow()).unwrap();
       
            //println!("JS code: `{js_code}`");
            //println!("Result: {:?}", result); // Should be Value::Number(42.0)
        });
        
        Ok(())
    }
}
