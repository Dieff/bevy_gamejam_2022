use super::wizard_memory::MemoryLocation;
use super::wizard_types::WizardScalarType;

enum Pointer {
  Static(MemoryLocation),
  Dynamic(MemoryLocation),
}

struct Value {
  DynamicPointer: Pointer,
  StaticType: WizardScalarType,
}

enum Function {
  Print(Value)
}

enum Line {
  Instruction,
  Comment
}

struct AST {
  lines: Vec<Line>
}
