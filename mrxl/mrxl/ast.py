from dataclasses import dataclass
from typing import List, Union


@dataclass
class Decl:
    input: bool  # Otherwise, output.
    name: str
    type: str  # TODO


@dataclass
class BinExpr:
    op: str
    lhs: "Expr"
    rhs: "Expr"


@dataclass
class LitExpr:
    value: int


@dataclass
class VarExpr:
    name: str


Expr = Union[BinExpr, LitExpr, VarExpr]


@dataclass
class Map:
    par: int
    bind: str  # TODO
    body: Expr


@dataclass
class Reduce:
    par: int
    bind: str  # TODO
    init: int
    body: Expr


@dataclass
class Stmt:
    dest: str
    op: Union[Map, Reduce]


@dataclass
class Prog:
    decls: List[Decl]
    stmts: List[Stmt]
