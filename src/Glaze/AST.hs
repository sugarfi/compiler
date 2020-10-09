module Glaze.AST where

data Type = TypeNumber
          | TypeString
          | TypeBool
          | TypeHex
          | TypeDimension
          | TypeEnum
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
          | ExprUnaryOp (String, Expr)
          | ExprBinaryOp (String, Expr, Expr)
          deriving (Show)

data Node = NodeSelector ([String], [Node])
          | NodeFunction (String, [String], [Node]) -- , [Type])
          | NodeDefinition (String, Expr) -- , Type)
          | NodeProp (String, [Expr])
          deriving (Show)
