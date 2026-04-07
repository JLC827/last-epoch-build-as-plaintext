use std::fs;
fn main() {
    let mut text = fs::read_to_string("src/sub_logic/resolver.rs").unwrap();
    text = text.replace("let mut subtype = self.get_base_type_name(tags).map(|s| s.clone()).unwrap_or_else(|| \"Unknown\".to_string());", "let subtype = self.get_base_type_name(tags).map(|s| s.clone()).unwrap_or_else(|| \"Unknown\".to_string());");
    fs::write("src/sub_logic/resolver.rs", text).unwrap();
}
