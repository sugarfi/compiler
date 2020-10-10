module Glaze.Generator where

import Glaze.AST
import Glaze.Util

import Data.List (intercalate)

isSelector :: Node -> Bool
isSelector (NodeSelector _) = True
isSelector _ = False

isProp :: Node -> Bool
isProp (NodeProp _) = True
isProp _ = False

-- Generation

generate :: [Node] -> [CSSNode]
generate = concatMap generateRootNode

generateRootNode :: Node -> [CSSNode]
generateRootNode (NodeSelector (sels, nodes)) =
    let
        props = map generateProp (filter isProp nodes)
        children = concatMap (generateNestedSelector sels) (filter isSelector nodes)
    in
    CSSSelector (sels, props) : children
-- generateRootNode (NodeFunction name args nodes types) =
-- generateRootNode (NodeDefinition name value) =

generateNestedSelector :: [String] -> Node -> [CSSNode]
generateNestedSelector parentSels (NodeSelector (sels, nodes)) =
    let
        nestedSels = map concatWithSpace $ combine parentSels sels
        props = map generateProp (filter isProp nodes)
        children = concatMap (generateNestedSelector nestedSels) (filter isSelector nodes)
    in
    CSSSelector (nestedSels, props) : children
    where
        concatWithSpace (a, b) = a ++ " " ++ b

generateProp :: Node -> (String, String)
generateProp (NodeProp (name, args)) =
    let
        value = unwords $ map exprToString args
    in
    (name, value)

-- Evaluation

evalExpr :: Expr -> Expr
evalExpr (ExprTuple t) =
    if length t == 1 then
        let (x:_) = t in evalExpr x
    else
        ExprTuple t
evalExpr (ExprBinaryOp (op, a, b)) = evalBinaryOp op (evalExpr a) (evalExpr b)
evalExpr (ExprUnaryOp (op, a)) = evalUnaryOp op (evalExpr a)
-- evalExpr (ExprVariable name) =
-- evalExpr (ExprFunction (name, args)) =
evalExpr expr = expr

evalBinaryOp :: String -> Expr -> Expr -> Expr
evalBinaryOp "and" (ExprBool a) (ExprBool b) = ExprBool (a && b)
evalBinaryOp "and" _ _ = error "Cannot use and"
evalBinaryOp "or" (ExprBool a) (ExprBool b) = ExprBool (a || b)
evalBinaryOp "or" _ _ = error "Cannot use or"
evalBinaryOp "+" (ExprNumber a) (ExprNumber b) = ExprNumber (a + b)
evalBinaryOp "+" _ _ = error "Cannot use +"
evalBinaryOp "-" (ExprNumber a) (ExprNumber b) = ExprNumber (a - b)
evalBinaryOp "-" _ _ = error "Cannot use -"
evalBinaryOp "*" (ExprNumber a) (ExprNumber b) = ExprNumber (a * b)
evalBinaryOp "*" _ _ = error "Cannot use *"
evalBinaryOp "/" (ExprNumber a) (ExprNumber b) = ExprNumber (a / b)
evalBinaryOp "/" _ _ = error "Cannot use /"

evalUnaryOp :: String -> Expr -> Expr
evalUnaryOp "not" (ExprBool a) = ExprBool (not a)
evalUnaryOp "not" _ = error "Cannot use not"

exprToString :: Expr -> String
exprToString (ExprNumber n) = if isInt n then show $ round n else show n
exprToString (ExprBool b) = if b then "true" else "false"
exprToString (ExprSymbol s) = s
exprToString (ExprHex h) = "#" ++ h
exprToString (ExprDimension (v, u)) = exprToString (ExprNumber v) ++ u
exprToString (ExprTuple t) = unwords $ map (exprToString . evalExpr) t
exprToString (ExprList l) = intercalate ", " $ map (exprToString . evalExpr) l
exprToString (ExprRecord _) = "record" -- temp
exprToString expr = exprToString $ evalExpr expr
