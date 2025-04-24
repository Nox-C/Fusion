use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ButtonProps {
    /// Additional CSS classes
    #[prop_or_default]
    pub class: String,
    /// Click callback
    #[prop_or_default]
    pub onclick: Option<Callback<MouseEvent>>,
    /// Button contents
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let ButtonProps { class, onclick, children } = props.clone();
    let callback = onclick.unwrap_or_else(Callback::noop);
    html! {
        <button class={class} onclick={callback}>{ for children.iter() }</button>
    }
}
