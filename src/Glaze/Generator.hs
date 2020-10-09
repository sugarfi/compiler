module Glaze.Generator where

import Glaze.AST

import Control.Applicative (liftA2)
import Data.List (intercalate)

isSelector :: Node -> Bool
isSelector (NodeSelector _) = True
isSelector _ = False

isProp :: Node -> Bool
isProp (NodeProp _) = True
isProp _ = False

-- Generation

generate :: [Node] -> [CSSNode]
generate nodes = concat $ map generateRootNode nodes

generateRootNode :: Node -> [CSSNode]
generateRootNode (NodeSelector (sels, nodes)) =
    let
        props = map generateProp (filter isProp nodes)
        children = concat $ map (generateNestedSelector sels) (filter isSelector nodes)
    in
    [CSSSelector (sels, props)] ++ children
-- generateRootNode (NodeFunction name args nodes types) =
-- generateRootNode (NodeDefinition name value) =

generateNestedSelector :: [String] -> Node -> [CSSNode]
generateNestedSelector parentSels (NodeSelector (sels, nodes)) =
    let
        nestedSels = map concatWithSpace $ liftA2 (,) parentSels sels
        props = map generateProp (filter isProp nodes)
        children = concat $ map (generateNestedSelector nestedSels) (filter isSelector nodes)
    in
    [CSSSelector (nestedSels, props)] ++ children
    where
        concatWithSpace (a, b) = a ++ " " ++ b

generateProp :: Node -> (String, String)
generateProp (NodeProp (name, args)) =
    let
        value = intercalate " " $ map exprToString args
    in
    (name, value)

-- Evaluation

evalExpr :: Expr -> Expr
-- evalExpr (ExprVariable name) =
-- evalExpr (ExprFunction (name, args)) =
-- evalExpr (ExprUnaryOp (op, a)) =
-- evalExpr (ExprBinaryOp (op, a, b)) =
evalExpr expr = expr

exprToString :: Expr -> String
exprToString (ExprNumber n) = show n
exprToString (ExprBool b) = show b
exprToString (ExprSymbol s) = s
exprToString (ExprHex h) = "#" ++ h
exprToString (ExprDimension (v, u)) = (show v) ++ u
exprToString (ExprTuple t) = intercalate " " $ map (exprToString . evalExpr) t
exprToString (ExprList l) = intercalate ", " $ map (exprToString . evalExpr) l
exprToString (ExprRecord _) = "record" -- temp
exprToString expr = exprToString $ evalExpr expr
