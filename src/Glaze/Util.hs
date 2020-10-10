module Glaze.Util where

import Control.Applicative (liftA2)

combine :: [a] -> [b] -> [(a, b)]
combine = liftA2 (,)

isInt :: Float -> Bool
isInt x = x == fromInteger (round x)
