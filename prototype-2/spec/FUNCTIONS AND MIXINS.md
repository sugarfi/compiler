# Functions and Mixins

## Function definition

```glaze
add(a, b) :: Number -> Number -> Number
	$a + $b
```

## Calling

```glaze
p
	font-weight: add(200, 400)
```

## Mixin

Mixins are simply functions that return props.

```glaze
color-weight(c, w) :: Hex -> Number -> Props
	color: $c
	font-weight: $w

p
	color-weight(#222, 400)
	# Or alternatively
	color-weight: #222 400
```

## Multiple Dispatch

Functions and mixins support multiple dispatch:

```glaze
add(a, b) :: Number -> Number -> Number
	$a + $b

add(a, b) :: String -> String -> String
	$a ++ $b
```
