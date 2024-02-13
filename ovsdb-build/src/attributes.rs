use std::ops::Deref;

#[derive(Clone, Debug, Default)]
pub(crate) struct Attributes(Vec<syn::Attribute>);

impl Attributes {
    pub fn new<S>(init: &[S]) -> Self
    where
        S: AsRef<str>,
    {
        let mut attrs = Self::default();
        for val in init {
            attrs.add(val);
        }
        attrs
    }

    pub fn add<S>(&mut self, value: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        let template = format!("{}\nstruct dummy;", value.as_ref());
        let mut attrs = syn::parse_str::<syn::DeriveInput>(&template).unwrap().attrs;
        self.0.append(&mut attrs);
        self
    }
}

impl Deref for Attributes {
    type Target = Vec<syn::Attribute>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
