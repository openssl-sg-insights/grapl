from typing import Any

from lark import Lark, Transformer


class HCL2TypeTransformer(Transformer):
    def hcl2_type(self, item: list) -> Any:
        return item[0]

    def value(self, item: list) -> Any:
        return item[0]

    def map(self, item: list) -> dict[str, Any]:
        return {"string": item[0]}

    def pair(self, key_value: tuple) -> tuple[str, Any]:
        k, v = key_value
        return k[1:-1], v

    list = list
    object = dict

    string_type = lambda self, _: "string"
    number_type = lambda self, _: "number"
    bool_type = lambda self, _: "bool"


class HCL2TypeParser:
    def __init__(self) -> None:
        self.parser = Lark(
            r"""
            hcl2_type: "${" value "}"
            value: list
                 | object
                 | map
                 | "string" -> string_type
                 | "number" -> number_type
                 | "bool" -> bool_type
        
            list : "[" [value ("," value)*] "]"
            map : "map(" value ")"
        
            object : "object({" [pair ("," pair)*] "})"
            pair : STRING_LITERAL ":" "'" hcl2_type "'"
            STRING_LITERAL : "'" _STRING_ESC_INNER "'"
            
            %import common._STRING_ESC_INNER
            %import common.WS
            %ignore WS
        """,
            start="hcl2_type",
            # Start speedup optimizations
            parser="lalr",
            # Disabling propagate_positions and placeholders slightly improves speed
            propagate_positions=False,
            maybe_placeholders=False,
            # Using an internal transformer is faster and more memory efficient
            transformer=HCL2TypeTransformer(),
        )
