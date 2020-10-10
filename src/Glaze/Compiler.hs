module Glaze.Compiler where

import Glaze.AST

import Data.List (intercalate)

compile :: [CSSNode] -> String
compile nodes =
    intercalate "\n\n" (filter (not . null) $ map compileNode nodes)
    ++ "\n"

compileNode :: CSSNode -> String
compileNode (CSSSelector (sels, props)) =
    if not (null props) then
        intercalate ",\n" sels ++ " {\n" ++ concatMap compileProp props ++ "}"
    else
        ""
    where
        compileProp (name, value) = "\t" ++ name ++ ": " ++ value ++ ";\n"
