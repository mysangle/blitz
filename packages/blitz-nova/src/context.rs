
use std::borrow::BorrowMut;

use nova_vm::{
    ecmascript::{
        execution::{
            agent::{Agent, GcAgent, HostHooks, Options, RealmRoot},
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

use crate::extension::recommended_extensions;

#[derive(Debug)]
pub enum JSContextError {
    
}

pub struct JSContext {
    pub agent: GcAgent,
    pub realm_root: RealmRoot,
}

impl JSContext {
    pub fn new<ScriptMacroTask: 'static>(host_hooks: &'static dyn HostHooks) -> Self {
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
                        extension.load::<ScriptMacroTask>(
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

pub const RUNTIME_JS: &str = r#"

blitzDocument = { querySelectorAll: function(s) {
    var handles = __blitz__.internal_query_selector_all(s);
    return handles.map(function(h) { return new Node(h) });
}}

LISTENERS = {}

function Node(handle) { this.handle = handle; }

Node.prototype.getAttribute = function(attr) {
    return __blitz__.internal_get_attribute(this.handle, attr);
}

Node.prototype.addEventListener = function(type, listener) {
    if (!LISTENERS[this.handle]) LISTENERS[this.handle] = {};
    var dict = LISTENERS[this.handle];
    if (!dict[type]) dict[type] = [];
    var list = dict[type];
    list.push(listener);
}

Node.prototype.dispatchEvent = function(evt) {
    var type = evt.type;
    var handle = this.handle;
    var list = (LISTENERS[handle] && LISTENERS[handle][type]) || [];
    for (var i = 0; i < list.length; i++) {
            list[i].call(this, evt);
        }
    return evt.do_default;
}

Object.defineProperty(Node.prototype, 'innerHTML', {
    set: function(s) {
        __blitz__.internal_inner_html_set(this.handle, s.toString());
    }
});

function Event(type) {
    this.type = type
    this.do_default = true;
}

Event.prototype.preventDefault = function() {
    this.do_default = false;
}

globalThis.document = blitzDocument;
"#;
