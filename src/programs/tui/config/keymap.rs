use std::{fmt::Debug, marker::PhantomData};

use crossterm::event::KeyEvent;
use serde::{de::Visitor, Deserialize, Deserializer};

use crate::programs::tui::config::keybinding::key_event;

use super::keybinding::{
    CollectionsKeyAction, GlobalKeyAction, KeyAction, KeyBinding, MethodUrlKeyAction,
    RequestBuilderKeyAction, TableKeyAction,
};

#[derive(Deserialize)]
pub struct KeyMap {
    #[serde(deserialize_with = "default_keybindings_deserializer")]
    #[serde(default = "KeyBinding::default_list")]
    global: Vec<KeyBinding<GlobalKeyAction>>,

    #[serde(deserialize_with = "default_keybindings_deserializer")]
    #[serde(default = "KeyBinding::default_list")]
    table: Vec<KeyBinding<TableKeyAction>>,

    #[serde(deserialize_with = "default_keybindings_deserializer")]
    #[serde(default = "KeyBinding::default_list")]
    method_url: Vec<KeyBinding<MethodUrlKeyAction>>,

    #[serde(deserialize_with = "default_keybindings_deserializer")]
    #[serde(default = "KeyBinding::default_list")]
    request_builder: Vec<KeyBinding<RequestBuilderKeyAction>>,

    #[serde(deserialize_with = "default_keybindings_deserializer")]
    #[serde(default = "KeyBinding::default_list")]
    collections: Vec<KeyBinding<CollectionsKeyAction>>,
}

impl KeyMap {
    fn _match<T: KeyAction + Copy>(list: &Vec<KeyBinding<T>>, key_event: KeyEvent) -> Option<T> {
        list.iter()
            .find(|key_binding| key_binding.key_event == key_event)
            .map(|key_binding| key_binding.action)
    }

    pub fn match_global_action(&self, key_event: KeyEvent) -> Option<GlobalKeyAction> {
        Self::_match(&self.global, key_event)
    }

    pub fn match_collections_action(&self, key_event: KeyEvent) -> Option<CollectionsKeyAction> {
        Self::_match(&self.collections, key_event)
    }

    pub fn match_table_action(&self, key_event: KeyEvent) -> Option<TableKeyAction> {
        Self::_match(&self.table, key_event)
    }

    pub fn match_method_url_action(&self, key_event: KeyEvent) -> Option<MethodUrlKeyAction> {
        Self::_match(&self.method_url, key_event)
    }

    pub fn match_request_builder_action(
        &self,
        key_event: KeyEvent,
    ) -> Option<RequestBuilderKeyAction> {
        Self::_match(&self.request_builder, key_event)
    }
}

fn default_keybindings_deserializer<'de, D, T>(
    deserializer: D,
) -> Result<Vec<KeyBinding<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: KeyAction + Deserialize<'de> + Debug + PartialEq,
{
    struct KeyBindingVisitor<T>(PhantomData<T>);

    impl<'vi, T> Visitor<'vi> for KeyBindingVisitor<T>
    where
        T: KeyAction + Deserialize<'vi> + Debug + PartialEq,
    {
        type Value = Vec<KeyBinding<T>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("Expect a map of Action = Key")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'vi>,
        {
            let mut key_bindings: Vec<KeyBinding<T>> = KeyBinding::default_list();

            while let Some((action, _key)) = map.next_entry::<T, String>()? {
                match key_event::str_to_key_event(&_key) {
                    // I should find the event, because the seed for the vec would have the all action default
                    Some(key_event) => {
                        let i = key_bindings
                            .iter()
                            .enumerate()
                            .find(|(_i, key_binding)| key_binding.action == action)
                            .map(|(i, _)| i)
                            .expect("The action should exist on the vec for");

                        key_bindings.swap_remove(i);
                        key_bindings.push(KeyBinding::new(action, key_event));
                    }
                    None => return Err(serde::de::Error::custom("")),
                }
            }

            Ok(key_bindings)
        }
    }

    deserializer.deserialize_map(KeyBindingVisitor(PhantomData))
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            global: KeyBinding::default_list(),
            table: KeyBinding::default_list(),
            method_url: KeyBinding::default_list(),
            request_builder: KeyBinding::default_list(),
            collections: KeyBinding::default_list(),
        }
    }
}
