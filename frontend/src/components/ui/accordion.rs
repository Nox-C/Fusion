use yew::prelude::*;

/// Root accordion container
#[derive(Properties, PartialEq)]
pub struct AccordionProps {
    pub children: Children,
}

#[function_component(Accordion)]
pub fn accordion(props: &AccordionProps) -> Html {
    html! { <div class="accordion-root">{ for props.children.iter() }</div> }
}

/// Single accordion item
#[derive(Properties, PartialEq)]
pub struct AccordionItemProps {
    pub children: Children,
    pub class: String,
}

#[function_component(AccordionItem)]
pub fn accordion_item(props: &AccordionItemProps) -> Html {
    html! {
        <details class={props.class.clone()}>
            { for props.children.iter() }
        </details>
    }
}

/// Accordion trigger (clickable header)
#[derive(Properties, PartialEq)]
pub struct AccordionTriggerProps {
    pub children: Children,
    pub class: String,
    pub on_toggle: Callback<bool>,
}

#[function_component(AccordionTrigger)]
pub fn accordion_trigger(props: &AccordionTriggerProps) -> Html {
    let details_ref = use_node_ref();
    let onclick = {
        let details_ref = details_ref.clone();
        let on_toggle = props.on_toggle.clone();
        Callback::from(move |_| {
            if let Some(details) = details_ref.cast::<web_sys::HtmlDetailsElement>() {
                on_toggle.emit(details.open());
            }
        })
    };
    html! {
        <summary class={props.class.clone()} onclick={onclick} ref={details_ref}>
            { for props.children.iter() }
        </summary>
    }
}

/// Accordion content (hidden until open)
#[derive(Properties, PartialEq)]
pub struct AccordionContentProps {
    pub children: Children,
    pub class: String,
}

#[function_component(AccordionContent)]
pub fn accordion_content(props: &AccordionContentProps) -> Html {
    html! { <div class={props.class.clone()}>{ for props.children.iter() }</div> }
}
