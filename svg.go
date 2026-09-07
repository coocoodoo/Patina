package patina

import (
	"errors"
	"image"

	"github.com/patina-ui/patina/internal/native"
)

// RenderSVG rasterizes SVG markup into an image of the given pixel size. The document is
// stretched to the requested size, so pass a size with the same aspect ratio as the SVG.
// `currentColor` resolves to the theme's text color.
func RenderSVG(svg string, width, height int) (*image.NRGBA, error) {
	return renderSVG(svg, width, height, ColorDefault)
}

// RenderSVGTinted is like RenderSVG but recolors every pixel with tint, keeping only the
// document's alpha. This turns any monochrome icon into the requested color.
func RenderSVGTinted(svg string, width, height int, tint Color) (*image.NRGBA, error) {
	return renderSVG(svg, width, height, tint)
}

func renderSVG(svg string, width, height int, tint Color) (*image.NRGBA, error) {
	mustInit()
	if width <= 0 || height <= 0 {
		return nil, errors.New("patina: RenderSVG needs a positive size")
	}
	pix, err := native.SVGRender(svg, width, height, int64(tint))
	if err != nil {
		return nil, err
	}
	return &image.NRGBA{Pix: pix, Stride: width * 4, Rect: image.Rect(0, 0, width, height)}, nil
}
