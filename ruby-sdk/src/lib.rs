use magnus::{define_module, function, method, prelude::*, Error};

use std::collections::HashMap;
use unleash_types::client_features::Context as InnerContext;
use unleash_yggdrasil::{EngineState, VariantDef};

#[magnus::wrap(class = "Unleash::Engine", free_immediatly, size)]
struct UnleashEngine {
    engine_state: EngineState,
}

#[magnus::wrap(class = "Unleash::Context", free_immediatly, size)]
pub struct Context {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub environment: Option<String>,
    pub app_name: Option<String>,
    pub current_time: Option<String>,
    pub remote_address: Option<String>,
    pub properties: Option<HashMap<String, String>>,
}

impl Context {
    pub fn new(
        user_id: Option<String>,
        session_id: Option<String>,
        remote_address: Option<String>,
        environment: Option<String>,
        app_name: Option<String>,
        current_time: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> Context {
        Context {
            user_id,
            session_id,
            remote_address,
            environment,
            app_name,
            properties,
            current_time,
        }
    }
}

#[magnus::wrap(class = "Unleash::Variant", free_immediatly, size)]
struct Variant {
    pub name: String,
    pub payload: Option<HashMap<String, String>>,
    pub enabled: bool,
}

impl From<VariantDef> for Variant {
    fn from(variant: VariantDef) -> Self {
        let mut payload = HashMap::new();
        if let Some(payload_content) = variant.payload {
            payload.insert("type".into(), payload_content.payload_type.clone());
            payload.insert("value".into(), payload_content.value);
        }
        Variant {
            name: variant.name,
            payload: Some(payload),
            enabled: variant.enabled,
        }
    }
}

impl From<&Context> for InnerContext {
    fn from(context_wrapper: &Context) -> Self {
        InnerContext {
            user_id: context_wrapper.user_id.clone(),
            session_id: context_wrapper.session_id.clone(),
            environment: context_wrapper.environment.clone(),
            app_name: context_wrapper.app_name.clone(),
            current_time: context_wrapper.current_time.clone(),
            remote_address: context_wrapper.remote_address.clone(),
            properties: context_wrapper.properties.clone(),
        }
    }
}

impl UnleashEngine {
    // #[new]
    pub fn default() -> UnleashEngine {
        UnleashEngine {
            engine_state: EngineState::default(),
        }
    }

    pub fn take_state(&mut self, state: String) {
        let toggles = serde_json::from_str(&state).expect("Failed to parse client spec");
        self.engine_state.take_state(toggles)
    }

    pub fn is_enabled(&self, name: String, context: &Context) -> bool {
        let context = context.into();
        self.engine_state.is_enabled(name, &context)
    }

    pub fn get_variant(&self, name: String, context: &Context) -> Variant {
        let context = context.into();
        self.engine_state.get_variant(name, &context).into()
    }
}

#[magnus::init]
fn init() -> Result<(), Error> {
    let module = define_module("Unleash")?;
    let context_class = module.define_class("Context", Default::default())?;
    context_class.define_singleton_method("new", function!(Context::new, 7))?;

    let engine_class = module.define_class("Engine", Default::default())?;
    engine_class.define_singleton_method("new", function!(UnleashEngine::default, 0))?;
    engine_class.define_method("take_state", method!(UnleashEngine::take_state, 1))?;
    engine_class.define_method("is_enabled", method!(UnleashEngine::is_enabled, 2))?;
    engine_class.define_method("get_variant", method!(UnleashEngine::get_variant, 2))?;

    let variant_class = module.define_class("Variant", Default::default())?;
    // variant_class.define_singleton_method("new", function!(Variant::default, 7))?;


    Ok(())
}
