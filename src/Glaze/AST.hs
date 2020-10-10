module Glaze.AST where

-- Glaze

data Type = TypeNumber
          | TypeString
          | TypeBool
          | TypeHex
          | TypeDimension
          | TypeEnum String
          | TypeTuple [Type]
          | TypeList Type
          | TypeRecord [(String, Type)]
          | TypeFunction [Type]
          | TypeProps
          deriving (Show)

data Expr = ExprNumber Float
          | ExprString String
          | ExprBool Bool
          | ExprSymbol String
          | ExprHex String
          | ExprDimension (Float, String)
          | ExprTuple [Expr]
          | ExprList [Expr]
          | ExprRecord [(String, Expr)]
          | ExprVariable String
          | ExprFunction (String, [Expr])
          | ExprBinaryOp (String, Expr, Expr)
          | ExprUnaryOp (String, Expr)
          deriving (Show)

data Node = NodeSelector ([String], [Node])
          | NodeFunction (String, [String], [Node], [String])
          | NodeDefinition (String, Expr)
          | NodeProp (String, [Expr])
          deriving (Show)

-- CSS

data CSSNode = CSSSelector ([String], [(String, String)])
             deriving (Show)

-- JavaScript
