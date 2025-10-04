
pub fn event_dispatch_js(node_id: usize, event_type: &str) -> String {
    format!("new Node({}).dispatchEvent(new Event('{}'))", node_id, event_type)
}

pub fn set_timeout_js(handle: usize) -> String {
    format!("__runSetTimeout({})", handle)
}

pub const RUNTIME_JS: &str = r#"
blitzConsole = { log: function(message) { __blitz__.internal_print(message + "\n"); } }

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

SET_TIMEOUT_REQUESTS = {}

function setTimeout(callback, time_delta) {
    var handle = Object.keys(SET_TIMEOUT_REQUESTS).length;
    SET_TIMEOUT_REQUESTS[handle] = callback;
    __blitz__.internal_set_timeout(handle, time_delta);
}

function __runSetTimeout(handle) {
    var callback = SET_TIMEOUT_REQUESTS[handle]
    callback();
}

globalThis.console = blitzConsole;
globalThis.document = blitzDocument;
"#;
