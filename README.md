# Glaze

Glaze is a functional reactive UI programming language that compiles to CSS and JavaScript. It was created to make web-based UI programming more declarative, modular and stress-free by offering a [Stylus](https://github.com/stylus/stylus)-like syntax, full-featured functional style and package management system. It also offers a set of reactive features that extends the scope of CSS styling far past its current state.

## Why Glaze?

Web technologies have always been a pain to work with. However, over the past decade, an abundance of powerful frontend frameworks, programming languages and templating engines have made working with HTML and JavaScript much easier. Yet there has been little development on the CSS side. CSS preprocessors such as SASS provide a lot of useful sugar but don't solve the core issue (CSS itself is flawed and unpredictable). UI toolkits such as Bootstrap are too inflexible and can require so much overriding and customization that you might as well just go back to manual CSS styling. The result of these limitations is that adding styles to your application can often take more effort and frustration than the development of the application itself, no matter what powerful toolkits you may be using.

Glaze hopes to solve these issues. It borrows a lot of concepts from CSS preprocessors while also providing direct support for updates during runtime. Its functional reactive style makes working with UI very declarative and predictable. In addition, its package management system helps make it expansive and utility-first, allowing UI frameworks to be more customizable. Instead of adding a million difficult to override classes to your HTML, you can simply install packages that provide custom CSS properties that can handle anything from fully customizable carousels to grid systems.

## Installation

`npm i -g glaze`

`yarn global add glaze`

## Basic Usage

Create a project:

`glaze init`

Install a Glaze package:

`glaze add lost`

Compile output to the current directory:

`glaze ui/style.glz .`

Learn more about the CLI tool [here](https://glaze.dev/docs).

## Features

- Usage with any framework!
- Feature-rich CSS preprocessing*
- Easy package management and modularity*
- Fully functional programming style*
- Reactive UI updates*
- Ability to redefine the entire CSS language itself*
- Linting and error reporting*
- Utility-first workflow encouraged
- `!important` is illegal
- Lightning fast compilation*

\* Not done yet

## Example

The following will make sure all images within the .example class always have a height equal to their width. When a button within .example is clicked, the padding of all sibling images is doubled.

```glaze
.example
	img
		@data
			pad: 5

		padding: &.data.padding ++ px
		width: &.data.padding * 20px
		height: &.width

	button
		@click
			& ~ img
				@data
					pad: &.data.pad * 2 ++ px
```

You can then register the above components like so:

```html
<div class="example">
	<img data-pad="5" />
	<button>Click Me!</button>
</div>
```

There is much more to Glaze, check out the full documentation [here](https://glaze.dev/docs).

## Plugins (Planned)

- [Webpack]()
- [PostCSS]()
- [Svelte]()

## Packages

Find a full list of available packages [here](https://glaze.dev/packages).

## Support Us
