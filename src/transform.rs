use iref::{Iri, IriBuf, IriRef, IriRefBuf};
use json_ld::{
    context::TermDefinitionRef,
    syntax::{is_keyword, Keyword},
    Process,
};
use std::borrow::Cow;

use crate::{
    keywords::{FIRST_CUSTOM_TERM_ID, KEYWORDS_MAP},
    CborObject, CborValue, Codecs, DecodeError, EncodeError, IdAllocator, IdMap, JsonObject,
    JsonValue,
};

pub trait TransformedValue: Sized {
    type Object;

    fn new_array(items: Vec<Self>) -> Self;

    fn new_object(object: Self::Object) -> Self;

    fn as_array(&self) -> Option<&[Self]>;

    fn as_object(&self) -> Option<&Self::Object>;

    fn force_as_array(&self, plural: bool) -> &[Self] {
        if plural {
            match self.as_array() {
                Some(array) => array,
                None => std::slice::from_ref(self),
            }
        } else {
            std::slice::from_ref(self)
        }
    }

    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }
}

impl TransformedValue for JsonValue {
    type Object = JsonObject;

    fn new_array(items: Vec<Self>) -> Self {
        Self::Array(items)
    }

    fn new_object(object: JsonObject) -> Self {
        Self::Object(object)
    }

    fn as_array(&self) -> Option<&[Self]> {
        self.as_array()
    }

    fn as_object(&self) -> Option<&JsonObject> {
        self.as_object()
    }

    fn is_array(&self) -> bool {
        self.is_array()
    }
}

impl TransformedValue for CborValue {
    type Object = CborObject;

    fn new_array(items: Vec<Self>) -> Self {
        Self::Array(items)
    }

    fn new_object(object: Self::Object) -> Self {
        Self::Map(object)
    }

    fn as_array(&self) -> Option<&[Self]> {
        self.as_array().map(Vec::as_slice)
    }

    fn as_object(&self) -> Option<&Self::Object> {
        self.as_map()
    }

    fn is_array(&self) -> bool {
        self.is_array()
    }
}

pub struct DuplicateKey<K>(pub K, pub K);

pub trait TransformedObject {
    type Key;
    type Value;

    fn new(entries: Vec<(Self::Key, Self::Value)>) -> Self;

    fn get_context(&self) -> Result<Option<&Self::Value>, DuplicateKey<Self::Key>>;

    fn entries(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)>;
}

impl TransformedObject for JsonObject {
    type Key = json_ld::syntax::object::Key;
    type Value = JsonValue;

    fn new(entries: Vec<(Self::Key, Self::Value)>) -> Self {
        entries.into_iter().collect()
    }

    fn get_context(&self) -> Result<Option<&Self::Value>, DuplicateKey<Self::Key>> {
        self.get_unique("@context")
            .map_err(|e| DuplicateKey(e.0.key.clone(), e.1.key.clone()))
    }

    fn entries(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
        self.entries().iter().map(|e| (&e.key, &e.value))
    }
}

impl TransformedObject for CborObject {
    type Key = CborValue;
    type Value = CborValue;

    fn new(entries: Vec<(Self::Key, Self::Value)>) -> Self {
        entries
    }

    fn get_context(&self) -> Result<Option<&Self::Value>, DuplicateKey<Self::Key>> {
        let mut entry: Option<(&CborValue, &CborValue)> = None;
        let context_id = KEYWORDS_MAP.get_id("@context").unwrap();
        let context_id_plural: ciborium::value::Integer = (context_id + 1).into();
        let context_id: ciborium::value::Integer = context_id.into();

        for (key, value) in self {
            if let &CborValue::Integer(i) = key {
                if i == context_id || i == context_id_plural {
                    if let Some(old_entry) = entry.take() {
                        return Err(DuplicateKey(old_entry.0.clone(), key.clone()));
                    }

                    entry = Some((key, value))
                }
            }
        }

        Ok(entry.map(|(_, value)| value))
    }

    fn entries(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)> {
        self.iter().map(|(key, value)| (key, value))
    }
}

pub struct MissingKeyTerm<T>(pub T);

pub struct UndefinedTerm(pub String);

pub struct ExpectedObject;

pub struct InvalidTypeKind;

pub trait Transformer {
    type Input: TransformedValue<Object = Self::InputObject>;
    type Output: TransformedValue<Object = Self::OutputObject>;

    type InputObject: TransformedObject<Key = Self::InputKey, Value = Self::Input>;
    type OutputObject: TransformedObject<Key = Self::OutputKey, Value = Self::Output>;

    type InputKey: ToOwned;
    type OutputKey: PartialOrd;

    type Loader: json_ld::Loader;
    type Error: From<json_ld::context_processing::Error>
        + From<ExpectedObject>
        + From<DuplicateKey<Self::InputKey>>
        + From<MissingKeyTerm<<Self::InputKey as ToOwned>::Owned>>
        + From<UndefinedTerm>
        + From<InvalidTypeKind>;

    fn context_iri_ref(&self, value: &Self::Input) -> Result<IriRefBuf, Self::Error>;

    fn context_id(&self, value: &Self::Input, iri_ref: &IriRef) -> Self::Output;

    fn term_key(&self, term: &str, plural: bool) -> Result<Self::OutputKey, Self::Error>;

    fn term_value(&self, term: &str) -> Result<Self::Output, Self::Error>;

    fn key_term<'a>(
        &'a self,
        key: &'a Self::InputKey,
        value: &Self::Input,
    ) -> Result<Option<(&'a str, bool)>, Self::Error>;

    fn required_key_term<'a>(
        &'a self,
        key: &'a Self::InputKey,
        value: &Self::Input,
    ) -> Result<(&'a str, bool), Self::Error> {
        self.key_term(key, value)?
            .ok_or_else(|| MissingKeyTerm(key.to_owned()))
            .map_err(Into::into)
    }

    fn value_term<'a>(
        &'a self,
        active_context: &json_ld::Context,
        value: &'a Self::Input,
    ) -> Result<Cow<'a, str>, Self::Error>;

    /// Use the id codec to transform an id value.
    fn transform_id(&self, value: &Self::Input) -> Result<Self::Output, Self::Error>;

    /// Use the vocab codec to transform an vocab value.
    fn transform_vocab(
        &self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error>;

    fn state_and_loader_mut(&mut self) -> (&mut TransformerState, &mut Self::Loader);

    #[allow(async_fn_in_trait)]
    async fn process_global_context<'c>(
        &mut self,
        active_context: &'c json_ld::Context,
        context_value: &Self::Input,
        propagate: bool,
    ) -> Result<(Self::Output, Cow<'c, json_ld::Context>), Self::Error> {
        match context_value.as_array() {
            Some(entries) => {
                let mut active_context = Cow::Borrowed(active_context);
                let mut cbor_entries = Vec::with_capacity(entries.len());

                for entry in entries {
                    let (cbor_value, new_active_context) = self
                        .process_global_context_entry(&active_context, entry, propagate)
                        .await?;

                    active_context = Cow::Owned(new_active_context);
                    cbor_entries.push(cbor_value)
                }

                Ok((Self::Output::new_array(cbor_entries), active_context))
            }
            None => {
                let (cbor_value, new_active_context) = self
                    .process_global_context_entry(active_context, context_value, propagate)
                    .await?;
                Ok((cbor_value, Cow::Owned(new_active_context)))
            }
        }
    }

    #[allow(async_fn_in_trait)]
    async fn process_global_context_entry(
        &mut self,
        active_context: &json_ld::Context,
        context_value: &Self::Input,
        propagate: bool,
    ) -> Result<(Self::Output, json_ld::Context), Self::Error> {
        // let context_iri_ref: IriRefBuf = context_value
        //     .as_str()
        //     .ok_or(EncodeError::InvalidContextEntry)?
        //     .parse()
        //     .map_err(|_| EncodeError::InvalidContextEntry)?;

        let context_iri_ref = self.context_iri_ref(context_value)?;
        let id = self.context_id(context_value, &context_iri_ref);
        let context = json_ld::syntax::Context::iri_ref(context_iri_ref);
        let new_active_context = self
            .process_context(active_context, &context, propagate)
            .await?;

        Ok((id, new_active_context))
    }

    #[allow(async_fn_in_trait)]
    async fn process_context(
        &mut self,
        active_context: &json_ld::Context,
        context: &json_ld::syntax::Context,
        propagate: bool,
    ) -> Result<json_ld::Context, Self::Error> {
        let (state, loader) = self.state_and_loader_mut();

        let result = context
            .process_with(
                &mut (),
                active_context,
                loader,
                None,
                json_ld::context_processing::Options {
                    propagate,
                    ..Default::default()
                },
            )
            .await?
            .into_processed();

        let mut keys: Vec<_> = result
            .definitions()
            .iter()
            .map(|d| d.term().as_str())
            .collect();
        keys.sort_unstable();

        // Allocate ids.
        for key in keys {
            if !is_keyword(key) {
                state.allocator.allocate(key);
            }
        }

        Ok(result)
    }

    #[allow(async_fn_in_trait)]
    async fn transform(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error> {
        match value.as_object() {
            Some(object) => self
                .transform_node(active_context, object)
                .await
                .map(Self::Output::new_object),
            None => Err(ExpectedObject.into()),
        }
    }

    #[allow(async_fn_in_trait)]
    async fn transform_node(
        &mut self,
        active_context: &json_ld::Context,
        object: &Self::InputObject,
    ) -> Result<Self::OutputObject, Self::Error> {
        // // Otherwise element is a map.
        // // If `active_context` has a `previous_context`, the active context is not
        // // propagated.
        // let mut active_context = Mown::Borrowed(active_context);
        // if let Some(previous_context) = active_context.previous_context() {
        // 	// If `from_map` is undefined or false, and `element` does not contain an entry
        // 	// expanding to `@value`, and `element` does not consist of a single entry
        // 	// expanding to `@id` (where entries are IRI expanded), set active context to
        // 	// previous context from active context, as the scope of a term-scoped context
        // 	// does not apply when processing new Object objects.
        // 	if !from_map
        // 		&& preliminary_value_entry.is_none()
        // 		&& !(element.len() == 1 && preliminary_id_entry.is_some())
        // 	{
        // 		active_context = Mown::Owned(previous_context.clone())
        // 	}
        // }

        let mut active_context = Cow::Borrowed(active_context);
        let mut result = Vec::new();

        // Embedded contexts.
        if let Some(context_value) = object.get_context()? {
            // Encode them.
            let cbor_key = self.term_key("@context", context_value.is_array())?;

            let (cbor_value, new_active_context) = self
                .process_global_context(&active_context, context_value, true)
                .await?;

            if let Cow::Owned(new_active_context) = new_active_context {
                active_context = Cow::Owned(new_active_context)
            }

            result.push((cbor_key, cbor_value));
        }

        // Find types.
        let mut types = Vec::new();
        for (key, value) in object.entries() {
            if let Some((term, plural)) = self.key_term(key, value)? {
                if is_alias(&active_context, term, Keyword::Type) {
                    for ty in value.force_as_array(plural) {
                        let ty_term = self.value_term(&active_context, ty)?;
                        types.push(ty_term.into_owned());
                    }
                }
            }
        }

        // Apply type-scoped contexts.
        types.sort_unstable();
        for ty in types {
            if let Some(def) = active_context.get(ty.as_str()) {
                if let Some(context) = def.context() {
                    active_context = Cow::Owned(
                        self.process_context(&active_context, context, false)
                            .await?,
                    );
                }
            }
        }

        // Sort entries.
        let mut sorted_entries = Vec::new();
        for (key, value) in object.entries() {
            let (key_term, plural) = self.required_key_term(key, value)?;
            let key_term = key_term.to_owned();

            if key_term == "@context" {
                continue;
            }

            let def = active_context.get(key_term.as_str());
            let id = self.term_key(&key_term, value.is_array())?;
            sorted_entries.push((key_term, plural, def, id, value));
        }
        sorted_entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Process entries.
        for (key_term, plural, def, cbor_key, value) in sorted_entries {
            if is_alias_with_def(&key_term, def, Keyword::Id) {
                let cbor_value = self.transform_id(value)?;
                result.push((cbor_key, cbor_value));

                continue;
            }

            if is_alias_with_def(&key_term, def, Keyword::Type) {
                let cbor_value = if plural {
                    let values = value.as_array().ok_or(InvalidTypeKind)?;
                    let mut cbor_values = Vec::with_capacity(values.len());

                    for value in values {
                        cbor_values.push(self.transform_vocab(&active_context, value)?);
                    }

                    Self::Output::new_array(cbor_values)
                } else {
                    self.transform_vocab(&active_context, value)?
                };

                result.push((cbor_key, cbor_value));

                continue;
            }

            let def = def.ok_or_else(|| UndefinedTerm(key_term.clone()))?;

            // Apply property-scoped context.
            let mut property_context = Cow::Borrowed(active_context.as_ref());
            if let Some(context) = def.context() {
                property_context =
                    Cow::Owned(self.process_context(&active_context, context, true).await?);
            }

            let values = value.force_as_array(plural);
            let mut cbor_values = Vec::with_capacity(values.len());

            let value_type = def.typ();

            for value in values {
                let cbor_value =
                    match self.transform_typed_value(&active_context, value, value_type)? {
                        Some(cbor_value) => cbor_value,
                        None => self.transform_object(&property_context, value).await?,
                    };

                cbor_values.push(cbor_value)
            }

            let cbor_value = if plural {
                Self::Output::new_array(cbor_values)
            } else {
                cbor_values.into_iter().next().unwrap()
            };

            result.push((cbor_key, cbor_value));
        }

        result.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        Ok(Self::OutputObject::new(result))
    }

    fn transform_typed_value(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
        type_: Option<&json_ld::Type<IriBuf>>,
    ) -> Result<Option<Self::Output>, Self::Error>;

    #[allow(async_fn_in_trait)]
    async fn transform_object(
        &mut self,
        active_context: &json_ld::Context,
        value: &Self::Input,
    ) -> Result<Self::Output, Self::Error>;
}

fn is_alias(active_context: &json_ld::Context, key: &str, keyword: Keyword) -> bool {
    key == keyword.into_str()
        || active_context.get_normal(key).is_some_and(|d| {
            d.value
                .as_ref()
                .is_some_and(|d| *d == json_ld::Term::Keyword(keyword))
        })
}

fn is_alias_with_def(key: &str, def: Option<TermDefinitionRef>, keyword: Keyword) -> bool {
    key == keyword.into_str()
        || def.is_some_and(|d| {
            d.value()
                .is_some_and(|d| *d == json_ld::Term::Keyword(keyword))
        })
}

pub struct TransformerState {
    pub context_map: IdMap,
    pub allocator: IdAllocator,
    pub codecs: Codecs,
}

impl TransformerState {
    pub fn new(context_map: IdMap, codecs: Codecs) -> Self {
        Self {
            context_map,
            allocator: IdAllocator::new(Some(&KEYWORDS_MAP), FIRST_CUSTOM_TERM_ID),
            codecs,
        }
    }

    pub fn encode_vocab_term(
        &self,
        active_context: &json_ld::Context,
        value: &str,
    ) -> Result<CborValue, EncodeError> {
        match self.allocator.encode_term(value, false) {
            Some(id) => Ok(CborValue::Integer(id.into())),
            None => {
                let mut expanded_value = Cow::Borrowed(value);

                // Decode CURIE term.
                if let Some((prefix, suffix)) = expanded_value.split_once(':') {
                    if let Some(prefix_def) = active_context.get(prefix) {
                        if prefix_def.prefix() {
                            let prefix_value = prefix_def
                                .value()
                                .ok_or(EncodeError::InvalidTermDefinition)?;
                            expanded_value = Cow::Owned(format!("{prefix_value}:{suffix}"))
                        }
                    }
                }

                match Iri::new(expanded_value.as_ref()) {
                    Ok(iri) => self.codecs.iri.encode(iri),
                    Err(_) => Ok(CborValue::Text(value.to_owned())),
                }
            }
        }
    }

    pub fn decode_vocab_term(
        &self,
        _active_context: &json_ld::Context,
        value: &CborValue,
    ) -> Result<String, DecodeError> {
        match value {
            CborValue::Integer(i) => {
                let id: u64 = (*i).try_into().map_err(|_| DecodeError::InvalidValue)?;

                self.allocator
                    .decode_term(id)
                    .ok_or_else(|| DecodeError::UndefinedCompressedTerm(value.clone()))
                    .map(|(s, _)| s.to_owned())
            }
            CborValue::Text(t) => Ok(t.clone()),
            other => self.codecs.iri.decode(other).map(IriBuf::into_string),
        }
    }
}
