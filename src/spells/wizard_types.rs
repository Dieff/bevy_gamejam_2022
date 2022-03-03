use bevy::utils::HashMap;

pub enum WizardScalarType {
  Integer(u32),
  Char(u32),
  Bool(bool)
}

pub enum WizardFieldType {
  Scalar(WizardScalarType),
  List((u16, WizardScalarType)),
  Tuple((u8, WizardScalarType))
}

struct WizardForm {
  name: String,
  fields: HashMap<String, WizardFieldType>
}

