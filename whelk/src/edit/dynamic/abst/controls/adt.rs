use crate::edit::{dynamic::abst::{AbstractDynamic, Color, Container, DynamicContext, Field, FieldFocus, FieldRead, FieldSetColor, FieldTriggersAppend, FieldTriggersRemove, HasContainer, HasField, HasInitializedField, HasStatic, Static, VStack, Wrapper}, zipper::Term};
use mincodec::MinCodec;

use super::ControlData;

#[derive(MinCodec, Clone, Default)]
pub struct AdtVariantData {
    name: String
}

#[derive(MinCodec, Clone, Default)]
pub struct AdtData {
    name: String,
    variants: Vec<AdtVariantData>
}

pub struct AdtVariant<T: HasContainer<Wrapper>> 
where
    T::Field: Container,
    <T::Field as Container>::Context: DynamicContext + HasField<Static> + HasField<String> 
{
    wrapper: T::Field,
    name: Option<<<T::Field as Container>::Context as HasField<String>>::Field>,
    prefix: Option<<<T::Field as Container>::Context as HasField<Static>>::Field>,
    data_name: String,
    new: bool,
}

struct VariantEvents {
    remove: bool,
    append: bool
}

impl<T: HasContainer<Wrapper>> AdtVariant<T>
where
    T::Field: Container,
    <T::Field as Container>::Context: DynamicContext + HasStatic + HasInitializedField<String>,
    <<T::Field as Container>::Context as HasField<String>>::Field: FieldSetColor + FieldFocus + FieldRead<Data = String> + FieldTriggersAppend + FieldTriggersRemove
{
    fn render(&mut self, context: &mut T) -> VariantEvents {
        let wrapper = &self.wrapper;
        let data_name = &self.data_name;

        let prefix = self.prefix.get_or_insert_with(|| {
            let wrapper = <T as HasField<Wrapper>>::field(context, wrapper).context();

            let field = <_ as HasField<Static>>::create_field(wrapper, Static(" ".into()));

            wrapper.append_field(field.handle().into());

            field
        }).handle();

        let new = self.new;

        let name = self.name.get_or_insert_with(|| {
            let wrapper = <T as HasField<Wrapper>>::field(context, wrapper).context();

            let field = <_ as HasField<String>>::create_field(wrapper, Some(data_name.clone()));
            <_ as HasField<String>>::field(wrapper, &field).set_color(Color::Reference);

            if new {
                <_ as HasField<String>>::field(wrapper, &field).focus();
            }

            wrapper.append_field_after(field.handle().into(), prefix);

            field
        });

        let wrapper = <T as HasField<Wrapper>>::field(context, wrapper).context();
        let name = <_ as HasField<String>>::field(wrapper, name);

        let append = name.trigger_append();
        let remove = name.trigger_remove();

        if let Some(name) = name.read() {
            self.data_name = name;
        }

        VariantEvents {
            append,
            remove
        }
    }

    fn new(wrapper: T::Field, data_name: String, new: bool) ->  Self {
        Self {
            wrapper,
            data_name,
            name: None,
            prefix: None,
            new
        }
    }

    fn data(&self) -> AdtVariantData {
        AdtVariantData {
            name: self.data_name.clone()
        }
    }
}

pub struct Adt<T: DynamicContext + HasContainer<VStack> + ?Sized>
where
    <T as HasField<VStack>>::Field: Container,
    <<T as HasField<VStack>>::Field as Container>::Context: HasContainer<Wrapper>,
    <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field: Container,
    <<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context: HasField<Static> + HasField<String>
{
    stack: Option<<T as HasField<VStack>>::Field>,
    sig_wrapper: Option<
        <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field,
    >,
    prefix: Option<<<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<Static>>::Field>,
    name: Option<<<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<String>>::Field>,
    variants: Vec<AdtVariant<<<T as HasField<VStack>>::Field as Container>::Context>>,
    data_name: String,
    add_variants: Vec<AdtVariantData>
}

impl<T: DynamicContext + HasContainer<VStack> + ?Sized> AbstractDynamic<T> for Adt<T>
where
    <T as HasField<VStack>>::Field: Container,
    <<T as HasField<VStack>>::Field as Container>::Context: HasContainer<Wrapper>,
    <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field: Container,
    <<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context: HasStatic + HasInitializedField<String>,
    <<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<String>>::Field: FieldRead<Data = String> + FieldFocus + FieldSetColor + FieldTriggersRemove + FieldTriggersAppend,
    <<<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context as HasField<Static>>::Field: FieldSetColor
{
    fn render(&mut self, context: &mut T) {
        let stack = self.stack.get_or_insert_with(|| {
            let field = <T as HasField<VStack>>::create_field(context, ());
            context.append_field(field.handle());
            field
        });

        let wrapper =  self.sig_wrapper.get_or_insert_with(|| {
            let stack = <T as HasField<VStack>>::field(context, stack).context();

            let field = HasField::create_field(stack, ());

            stack.append_field(field.handle());
          
            field
        });

        let wrapper_handle = wrapper.handle();

        let prefix = self
            .prefix
            .get_or_insert_with(|| {
                let stack = <T as HasField<VStack>>::field(context, stack).context();
                let wrapper = <_ as HasField<Wrapper>>::field(stack, wrapper).context();

                let field = <_ as HasField<Static>>::create_field(wrapper, Static("𝑇".into()));
                <_ as HasField<Static>>::field(wrapper, &field).set_color(Color::Type);

                wrapper.append_field(field.handle().into());

                field

            })
            .handle();

        let name = self.data_name.clone();

        let name = self.name.get_or_insert_with(|| {
            let stack = <T as HasField<VStack>>::field(context, stack).context();
            let wrapper = <_ as HasField<Wrapper>>::field(stack, wrapper).context();

            let field = <_ as HasField<String>>::create_field(wrapper, Some(name));
            <_ as HasField<String>>::field(wrapper, &field).set_color(Color::Reference);

            wrapper.append_field(field.handle().into());

            field
        });

        let s = stack;
        let w = wrapper;
        let n = name;


        let stack = <T as HasField<VStack>>::field(context, s).context();
        let wrapper = <_ as HasField<Wrapper>>::field(stack, w).context();

        let name = <_ as HasField<String>>::field(wrapper, &*n);

        if let Some(data) = name.read() {
            self.data_name = data;
        }

        if name.trigger_append() {
            let field = HasField::create_field(stack, ());
            let handle = field.handle();
          
            self.variants.push(AdtVariant::new(field, "".into(), true));

            stack.append_field_after(handle, wrapper_handle);
        }

        let stack = <T as HasField<VStack>>::field(context, s).context();
        let wrapper = <_ as HasField<Wrapper>>::field(stack, w).context();

        let name = <_ as HasField<String>>::field(wrapper, &*n);

        if name.trigger_remove() {
            context.remove();
        }

        let stack = <T as HasField<VStack>>::field(context, s).context();

        let mut new_variants = vec![];

        let mut removed_idx = None;

        for variant in self.add_variants.drain(..) {
            let field = HasField::create_field(stack, ());
            let handle = field.handle();
          
            new_variants.push(AdtVariant::new(field, variant.name, false));

            stack.append_field(handle);
        }

        for (idx, variant) in self.variants.iter_mut().enumerate() {
            let events = variant.render(stack);

            if events.append {
                let field = HasField::create_field(stack, ());
                let handle = field.handle();
          
                new_variants.push(AdtVariant::new(field, "".into(), true));

                stack.append_field_after(handle, variant.wrapper.handle());
            } else if events.remove {
                let wrapper = <_ as HasField<Wrapper>>::field(stack, &variant.wrapper).context();
                wrapper.remove();
                removed_idx = Some(idx);
            }
        }

        if let Some(idx) = removed_idx {
            if idx > 0 {
                let variant = &self.variants[idx - 1];
                let wrapper = <_ as HasField<Wrapper>>::field(stack, &variant.wrapper).context();
                let name = <_ as HasField<String>>::field(wrapper, variant.name.as_ref().unwrap());
                name.focus();
            } else {
                let stack = <T as HasField<VStack>>::field(context, s).context();
                let wrapper = <_ as HasField<Wrapper>>::field(stack, w).context();
                let name = <_ as HasField<String>>::field(wrapper, &*n);
                name.focus();
            }

            self.variants.remove(idx);
        }

        self.variants.extend(new_variants);
    }

    fn expand(&self) -> Term<()> {
        Term::Reference(self.data_name.clone(), ())
    }

    fn encode(&self) -> ControlData {
        ControlData::Adt(AdtData {
            name: self.data_name.clone(),
            variants: self.variants.iter().map(AdtVariant::data).collect()
        })
    }
}

impl<T: DynamicContext + HasContainer<VStack> + ?Sized> Adt<T>
where
    <T as HasField<VStack>>::Field: Container,
    <<T as HasField<VStack>>::Field as Container>::Context: HasContainer<Wrapper>,
    <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field: Container,
    <<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context: HasField<Static> + HasField<String>
{
    pub fn new() -> Self {
        Adt::from(AdtData::default())
    }
}

impl<T: DynamicContext + HasContainer<VStack> + ?Sized> From<AdtData> for Adt<T>
where
    <T as HasField<VStack>>::Field: Container,
    <<T as HasField<VStack>>::Field as Container>::Context: HasContainer<Wrapper>,
    <<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field: Container,
    <<<<T as HasField<VStack>>::Field as Container>::Context as HasField<Wrapper>>::Field as Container>::Context: HasField<Static> + HasField<String>
{
    fn from(data: AdtData) -> Self {
        Self {
            stack: None,
            prefix: None,
            data_name: data.name,
            name: None,
            sig_wrapper: None,
            variants: vec![],
            add_variants: data.variants
        }
    }
}
