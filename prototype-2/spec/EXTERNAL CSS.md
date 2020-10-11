# External CSS

You can link to external CSS not handled by Glaze with the @css rule.

This is how Glaze defines its prelude. For example:

```glaze
color(c) :: Hex -> Props
	@css "color" c

enum Color = blue
           | red
           | green
           | ...

color(c) :: Color -> Props
	@css "color" c
```

Normally a prop is an implicit function call, so if a prop is not pre-defined it will result in an error.

`@css` allows you to specify props that don't have a definition.
