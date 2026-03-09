[
  "def"
  "struct"
  "interface"
  "event"
  "enum"
  "flag"
  "implements"
  "exports"
  "uses"
  "initializes"
  "if"
  "elif"
  "else"
  "for"
  "in"
  "return"
  "assert"
  "raise"
  "pass"
  "break"
  "continue"
  "import"
  "from"
  "as"
  "or"
  "and"
  "not"
  "log"
  "extcall"
  "staticcall"
  "view"
  "pure"
  "nonpayable"
  "payable"
  "public"
  "constant"
  "immutable"
  "transient"
  "reentrant"
  "indexed"
] @keyword

(comment) @comment
(pragma_directive) @preproc
(string) @string
(integer) @number
(boolean) @boolean

(module_docstring
  (docstring) @comment.doc)

(docstring_statement
  (docstring) @comment.doc)

[
  "="
  "=="
  "!="
  "<"
  "<="
  ">"
  ">="
  "+"
  "-"
  "*"
  "**"
  "//"
  "/"
  "%"
  "~"
  ":="
  "+="
  "-="
  "*="
  "/="
  "%="
  "|"
  "^"
  "&"
  "<<"
  ">>"
  "->"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  "."
  ","
  ":"
  "@"
] @punctuation.delimiter

(decorator
  name: (identifier) @attribute)

[
  (struct_declaration
    name: (identifier) @type)
  (interface_declaration
    name: (identifier) @type)
  (event_declaration
    name: (identifier) @type)
  (enum_declaration
    name: (identifier) @enum)
  (flag_declaration
    name: (identifier) @enum)
] 

[
  (state_variable_declaration
    name: (identifier) @property)
  (struct_member
    name: (identifier) @property)
  (event_member
    name: (identifier) @property)
  (module_binding
    name: (identifier) @variable)
] 

(constant_declaration
  name: (identifier) @constant)

[
  (function_signature
    name: (identifier) @function)
  (call
    function: (_) @function)
] 

[
  (parameter
    name: (identifier) @variable.parameter)
  (typed_loop_variable
    name: (identifier) @variable.parameter)
] 

(attribute
  attribute: (identifier) @property)

[
  (import_item
    (dotted_name
      (identifier) @module))
  (from_import_statement
    (relative_import_path
      (dotted_name
        (identifier) @module)))
  (imported_type
    (identifier) @module)
  (module_initialization
    module: (identifier) @module)
  (module_initialization
    module: (imported_type
      (identifier) @module))
] 

[
  (parameter
    type: (_) @type)
  (typed_loop_variable
    type: (_) @type)
  (function_signature
    return_type: (_) @type)
  (struct_member
    type: (_) @type)
  (event_member
    type: (_) @type)
  (state_variable_declaration
    type: (_) @type)
  (type_call
    (identifier) @type)
  (subscripted_type
    (identifier) @type)
  (hash_map_type
    (identifier) @type)
] 

((identifier) @variable.special
  (#any-of? @variable.special "self" "msg" "block" "tx" "chain"))
