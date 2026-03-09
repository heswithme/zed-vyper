(comment)+ @comment.around

(function_definition
  (block) @function.inside) @function.around

(struct_declaration) @class.around
(interface_declaration) @class.around
(event_declaration) @class.around
(enum_declaration) @class.around
(flag_declaration) @class.around
