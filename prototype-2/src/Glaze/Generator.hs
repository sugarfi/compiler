module Glaze.Generator where

import Glaze.AST
import Glaze.Util

import Control.Monad.State
import Data.List (find, intercalate)
import Text.Read (readMaybe)

isProp :: Node -> Bool
isProp (NodeProp _) = True
isProp _ = False

-- State

data GenState = GenState
    { css   :: [CSSNode]
    , js    :: [JSNode]
    , vars  :: [(String, Expr)]
    , funcs :: [(String, [String], [Node], [String])]
    }

findVar :: String -> State GenState Expr
findVar name = do
    state <- get
    let expr = case find (\(vname, _) -> vname == name) (vars state) of
                   Just (_, expr) -> expr
                   Nothing -> error ("Could not find variable: " ++ name)
    return expr

findFunc :: String -> [Expr] -> State GenState ([Node], [String])
findFunc name args = do
    state <- get
    let (_, params, nodes, types) = case find (\(fname, _, _, _) -> fname == name) (funcs state) of
                                        Just func -> func
                                        Nothing -> error ("Could not find function: " ++ name)
    args <- traverse evalExpr args
    let scope = if length args > length params then
                    error ("Too many arguments in function: " ++ name)
                else
                    zip params args

    put (state { vars = vars state ++ scope })
    return (nodes, types)

-- Nodes

generate :: [Node] -> ([CSSNode], [JSNode])
generate nodes = 
    let (_, state) = runState
                     (traverse generateRootNode nodes)
                     ( GenState { css   = []
                                , js    = []
                                , vars  = []
                                , funcs = []
                                }
                     )
    -- in (css state, js state)
    in (css state, js state)

generateRootNode :: Node -> State GenState ()
generateRootNode (NodeSelector (sels, nodes)) = do
    state <- get
    traverse (generateNode sels) (filter (not . isProp) nodes)
    props <- traverse generateProp (filter isProp nodes)
    put (state { css = CSSSelector (sels, props) : css state })
generateRootNode (NodeFunction func) = do
    state <- get
    put (state { funcs = funcs state ++ [func] })
generateRootNode (NodeDefinition var) = do
    state <- get
    put (state { vars = vars state ++ [var] })

generateNode :: [String] -> Node -> State GenState ()
generateNode sels (NodeAtRule (rule, nodes)) = do
    -- children <- concat <$> traverse (generateNestedSelector sels) (filter isSelector nodes)
    -- traverse (jsEvent rule) []
    return ()
generateNode parentSels (NodeSelector (sels, nodes)) = do
    state <- get
    let nestedSels = map concatWithSpace $ combine parentSels sels
    traverse (generateNode nestedSels) (filter (not . isProp) nodes)
    props <- traverse generateProp (filter isProp nodes)
    put (state { css = CSSSelector (nestedSels, props) : css state })
    where
        concatWithSpace (a, b) = a ++ " " ++ b

generateProp :: Node -> State GenState (String, String)
generateProp (NodeProp (name, args)) = do
    (nodes, types) <- findFunc name args
    props <- traverse generateProp (filter isProp nodes)
    value <- head <$> traverse exprToCSS args -- temp
    return (name, value)

-- JS

jsEvent :: String -> CSSNode -> State GenState ()
jsEvent event (CSSSelector (sels, props)) = do
    state <- get
    let updates = map update props
    let node = JSEvent (sels, event, updates)
    put (state { js = js state ++ [node] })
    where
        update (name, value) =
            let s = case readMaybe value :: Maybe Float of
                        Just _  -> value
                        Nothing -> "\"" ++ value ++ "\""
            in (name, s)

-- Expressions

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
evalExpr (ExprVariable name) = findVar name
evalExpr (ExprFunction (name, args)) = do
    (nodes, types) <- findFunc name args
    let (NodeExpr expr) = head nodes -- temp
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

exprToCSS :: Expr -> State GenState String
exprToCSS (ExprNumber n) = return $ numToString n
exprToCSS (ExprString s) = return ("\"" ++ s ++ "\"")
exprToCSS (ExprBool b) = if b then return "true" else return "false"
exprToCSS (ExprSymbol s) = return s
exprToCSS (ExprHex h) = return ("#" ++ h)
exprToCSS (ExprDimension (v, u)) = return (numToString v ++ u)
exprToCSS (ExprTuple _) = error "Tuple not valid in @css"
exprToCSS (ExprList _) = error "List not valid in @css"
exprToCSS (ExprRecord _) = error "Record not valid in @css"
exprToCSS expr = do
    expr <- evalExpr expr
    exprToCSS expr

numToString :: Float -> String
numToString n = if isInt n then
                    show $ round n
                else
                    show n
