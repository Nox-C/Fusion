use yew::Classes;

/// Merge multiple class fragments into a single `Classes`
/// 
/// Example:
/// ```rust
/// let classes = cn(vec!["px-4".into(), "bg-primary".into()]);
/// ```
pub fn cn<C: Into<Classes>>(inputs: Vec<C>) -> Classes {
    inputs.into_iter().fold(Classes::new(), |mut acc, c| {
        acc.extend(c.into());
        acc
    })
}
