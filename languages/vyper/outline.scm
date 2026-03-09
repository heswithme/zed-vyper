(decorator) @annotation

(function_definition
  (function_signature
    name: (identifier) @name)) @item

(function_definition
  (function_signature
    return_type: (_) @context.extra))

[
  (struct_declaration
    name: (identifier) @name)
  (interface_declaration
    name: (identifier) @name)
  (event_declaration
    name: (identifier) @name)
  (enum_declaration
    name: (identifier) @name)
  (flag_declaration
    name: (identifier) @name)
  (constant_declaration
    name: (identifier) @name)
  (state_variable_declaration
    name: (identifier) @name)
] @item
