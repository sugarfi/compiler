# Standard

## Prelude

The prelude includes typed definitions for all CSS properties and functions.

## Higher-order functions

Example of map function

```glaze
$arr = [1, 2, 3, 4]

addOne(n) :: Number -> Number
	$n + 1

p
	custom-prop: map(n -> $n + 1, $arr)
	# Or alternatively
	custom-prop: map(addOne, $arr)
```

Outputs to

```css
p {
	custom-prop: 2, 3, 4, 5;
}
```
