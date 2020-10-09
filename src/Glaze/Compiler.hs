module Glaze.Compiler where

import Glaze.AST

import Data.List (intercalate)

compile :: [CSSNode] -> String
compile nodes =
    ( intercalate "\n\n" $ 
      filter (\x -> length x > 0) $ 
      map compileNode nodes
    )
    ++ "\n"

compileNode :: CSSNode -> String
compileNode (CSSSelector (sels, props)) =
    if length props > 0 then
        intercalate ",\n" sels ++ " {\n" ++ (concat $ map compileProp props) ++ "}"
    else
        ""
    where
        compileProp (name, value) = "\t" ++ name ++ ": " ++ value ++ ";\n"
