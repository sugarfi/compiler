module Glaze.Generator where

import Glaze.AST
import Glaze.Util

import Control.Monad.State
import Data.List (find, intercalate)

isSelector :: Node -> Bool
isSelector (NodeSelector _) = True
isSelector _ = False

isProp :: Node -> Bool
isProp (NodeProp _) = True
isProp _ = False

-- State

data GenState = GenState
    { vars  :: [(String, Expr)]
    , funcs :: [(String, [String], [Node], [String])]
    }
    deriving (Show)

-- Generation

generate :: [Node] -> [CSSNode]
generate nodes = 
    let (css, _) = runState
                   (concat <$> traverse generateRootNode nodes)
                   (GenState { vars = [], funcs = [] })
    in css

generateRootNode :: Node -> State GenState [CSSNode]
generateRootNode (NodeSelector (sels, nodes)) = do
    (props, children) <- generateCSSSelector sels nodes
    return $ CSSSelector (sels, props) : children
generateRootNode (NodeFunction func) = do
    state <- get
    put (state { funcs = func : funcs state })
    return []
generateRootNode (NodeDefinition var) = do
    state <- get
    put (state { vars = var : vars state })
    return []

generateNestedSelector :: [String] -> Node -> State GenState [CSSNode]
generateNestedSelector parentSels (NodeSelector (sels, nodes)) = do
    let nestedSels = map concatWithSpace $ combine parentSels sels
    (props, children) <- generateCSSSelector nestedSels nodes
    return $ CSSSelector (nestedSels, props) : children
    where
        concatWithSpace (a, b) = a ++ " " ++ b

generateCSSSelector :: [String] -> [Node] -> State GenState ([(String, String)], [CSSNode])
generateCSSSelector sels nodes = do
    props <- traverse generateProp (filter isProp nodes)
    children <- concat <$> traverse (generateNestedSelector sels) (filter isSelector nodes)
    return (props, children)

generateProp :: Node -> State GenState (String, String)
generateProp (NodeProp (name, args)) = do
    value <- unwords <$> traverse exprToString args
    return (name, value)

-- Evaluation

evalExpr :: Expr -> State GenState Expr
evalExpr (ExprTuple t) =
    if length t == 1 then
        evalExpr $ head t
    else
        return $ ExprTuple t
evalExpr (ExprBinaryOp (op, a, b)) = do
    a <- evalExpr a
    b <- evalExpr b
    evalBinaryOp op a b
evalExpr (ExprUnaryOp (op, a)) = do
    a <- evalExpr a
    evalUnaryOp op a
evalExpr (ExprVariable name) = do
    state <- get
    let expr = case find (\(vname, _) -> vname == name) (vars state) of
                    Just (_, expr) -> expr
                    Nothing -> error ("Could not find variable: " ++ name)
    return expr
evalExpr (ExprFunction (name, args)) = do
    state <- get
    let (_, params, nodes, types) = case find (\(fname, _, _, _) -> fname == name) (funcs state) of
                                        Just func -> func
                                        Nothing -> error ("Could not find function: " ++ name)
    args <- traverse evalExpr args
    let scope = if length args > length params then
                    error ("Too many arguments in function: " ++ name)
                else
                    zip params args
    let (NodeExpr expr) = head nodes
    put (state { vars = scope ++ vars state })
    evalExpr expr
evalExpr expr = return expr

evalBinaryOp :: String -> Expr -> Expr -> State GenState Expr
evalBinaryOp "and" (ExprBool a) (ExprBool b) = return $ ExprBool (a && b)
evalBinaryOp "and" _ _ = error "Cannot use and"
evalBinaryOp "or" (ExprBool a) (ExprBool b) = return $ ExprBool (a || b)
evalBinaryOp "or" _ _ = error "Cannot use or"
evalBinaryOp "+" (ExprNumber a) (ExprNumber b) = return $ ExprNumber (a + b)
evalBinaryOp "+" _ _ = error "Cannot use +"
evalBinaryOp "-" (ExprNumber a) (ExprNumber b) = return $ ExprNumber (a - b)
evalBinaryOp "-" _ _ = error "Cannot use -"
evalBinaryOp "*" (ExprNumber a) (ExprNumber b) = return $ ExprNumber (a * b)
evalBinaryOp "*" _ _ = error "Cannot use *"
evalBinaryOp "/" (ExprNumber a) (ExprNumber b) = return $ ExprNumber (a / b)
evalBinaryOp "/" _ _ = error "Cannot use /"

evalUnaryOp :: String -> Expr -> State GenState Expr
evalUnaryOp "not" (ExprBool a) = return $ ExprBool (not a)
evalUnaryOp "not" _ = error "Cannot use not"

exprToString :: Expr -> State GenState String
exprToString (ExprNumber n) = if isInt n then return $ show $ round n else return $ show n
exprToString (ExprString s) = return ("\"" ++ s ++ "\"")
exprToString (ExprBool b) = if b then return "true" else return "false"
exprToString (ExprSymbol s) = return s
exprToString (ExprHex h) = return ("#" ++ h)
exprToString (ExprDimension (v, u)) = fmap (++ u) (exprToString (ExprNumber v))
exprToString (ExprTuple t) = unwords <$> traverse exprToString t
exprToString (ExprList l) = intercalate ", " <$> traverse exprToString l
exprToString (ExprRecord _) = return "record" -- temp
exprToString expr = do
    expr <- evalExpr expr
    exprToString expr
