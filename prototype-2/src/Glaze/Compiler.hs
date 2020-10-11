module Glaze.Compiler where

import Glaze.AST

import Data.List (intercalate)
import Text.Printf (printf)

compile :: ([CSSNode], [JSNode]) -> (String, String)
compile (css, js) =
    let
        outCSS = compileNodes compileCSSNode css
        outJS  = compileNodes compileJSNode  js
    in
    (outCSS, outJS)
    where
        compileNodes f nodes = intercalate "\n\n" (filter (not . null) $ map f nodes) ++ "\n"

-- CSS

compileCSSNode :: CSSNode -> String
compileCSSNode (CSSSelector (sels, props)) =
    if not (null props) then
        intercalate ",\n" sels ++ " {\n" ++ concatMap compileProp props ++ "}"
    else
        ""
    where
        compileProp (name, value) = "\t" ++ name ++ ": " ++ value ++ ";\n"

-- JS

compileJSNode :: JSNode -> String
compileJSNode (JSEvent (sels, event, nodes)) =
    let
        sel = intercalate ", " sels
    in
    if head sel == '#' then
        printf "document.getElementById(\"%s\").on%s = function() {};" (drop 1 sel) event
    else
        printf
        "for (const el of document.querySelector(\"%s\")) {\n\
        \    el.on%s = function() {};\n\
        \}"
        sel event
