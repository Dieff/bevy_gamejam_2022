use std::rc::Rc;

use crate::wizard_memory::MemoryLocation;

enum Pointer {
  Static(MemoryLocation),
  Dynamic(MemoryLocation),
}

struct Value {
  DynamicPointer: Pointer,
  StaticType: crate::wizard_types::WizardScalarType,
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
