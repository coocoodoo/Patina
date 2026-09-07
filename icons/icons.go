// Package icons bundles a small set of line icons as SVG strings for use with patina.Icon
// and Button.Icon:
//
//	patina.Button("Save").Icon(icons.Download)
//	patina.Icon(icons.Search).Size(20, 20).Tint(patina.TextSecondary)
//
// Every icon is drawn on a 24x24 grid with a 2px round stroke in currentColor, so it takes
// the theme's text color (or any Tint) automatically.
package icons

const head = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">`
const tail = `</svg>`

const (
	Check        = head + `<polyline points="20 6 9 17 4 12"/>` + tail
	Close        = head + `<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>` + tail
	Plus         = head + `<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>` + tail
	Minus        = head + `<line x1="5" y1="12" x2="19" y2="12"/>` + tail
	Menu         = head + `<line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>` + tail
	Search       = head + `<circle cx="11" cy="11" r="7"/><line x1="21" y1="21" x2="16.2" y2="16.2"/>` + tail
	ChevronLeft  = head + `<polyline points="15 18 9 12 15 6"/>` + tail
	ChevronRight = head + `<polyline points="9 18 15 12 9 6"/>` + tail
	ChevronUp    = head + `<polyline points="18 15 12 9 6 15"/>` + tail
	ChevronDown  = head + `<polyline points="6 9 12 15 18 9"/>` + tail
	ArrowLeft    = head + `<line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/>` + tail
	ArrowRight   = head + `<line x1="5" y1="12" x2="19" y2="12"/><polyline points="12 5 19 12 12 19"/>` + tail
	ArrowUp      = head + `<line x1="12" y1="19" x2="12" y2="5"/><polyline points="5 12 12 5 19 12"/>` + tail
	ArrowDown    = head + `<line x1="12" y1="5" x2="12" y2="19"/><polyline points="19 12 12 19 5 12"/>` + tail
	Info         = head + `<circle cx="12" cy="12" r="9"/><line x1="12" y1="11" x2="12" y2="16"/><line x1="12" y1="8" x2="12" y2="8.01"/>` + tail
	Warning      = head + `<path d="M12 3 21.5 20h-19z"/><line x1="12" y1="10" x2="12" y2="14"/><line x1="12" y1="17" x2="12" y2="17.01"/>` + tail
	Trash        = head + `<polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14H6L5 6"/><path d="M9 6V4h6v2"/><line x1="10" y1="11" x2="10" y2="16"/><line x1="14" y1="11" x2="14" y2="16"/>` + tail
	Edit         = head + `<path d="M17 3l4 4L8 20H4v-4L17 3z"/>` + tail
	Star         = head + `<polygon points="12 2.5 15 8.8 22 9.7 17 14.5 18.2 21.4 12 18.1 5.8 21.4 7 14.5 2 9.7 9 8.8"/>` + tail
	Heart        = head + `<path d="M12 20.5C7 16 3 13 3 8.5A4.5 4.5 0 0 1 12 6.5a4.5 4.5 0 0 1 9 2c0 4.5-4 7.5-9 12z"/>` + tail
	Home         = head + `<path d="M3 11l9-8 9 8"/><path d="M5 10v10h14V10"/><path d="M10 20v-6h4v6"/>` + tail
	User         = head + `<circle cx="12" cy="8" r="4"/><path d="M4 21c0-4 3.6-6.5 8-6.5s8 2.5 8 6.5"/>` + tail
	Bell         = head + `<path d="M6 16v-5a6 6 0 0 1 12 0v5l2 2H4z"/><path d="M10 21a2 2 0 0 0 4 0"/>` + tail
	Sun          = head + `<circle cx="12" cy="12" r="4"/><line x1="12" y1="2" x2="12" y2="4"/><line x1="12" y1="20" x2="12" y2="22"/><line x1="2" y1="12" x2="4" y2="12"/><line x1="20" y1="12" x2="22" y2="12"/><line x1="4.9" y1="4.9" x2="6.3" y2="6.3"/><line x1="17.7" y1="17.7" x2="19.1" y2="19.1"/><line x1="4.9" y1="19.1" x2="6.3" y2="17.7"/><line x1="17.7" y1="6.3" x2="19.1" y2="4.9"/>` + tail
	Moon         = head + `<path d="M21 13A9 9 0 1 1 11 3a7 7 0 0 0 10 10z"/>` + tail
	Settings     = head + `<line x1="4" y1="6" x2="20" y2="6"/><line x1="4" y1="12" x2="20" y2="12"/><line x1="4" y1="18" x2="20" y2="18"/><circle cx="9" cy="6" r="2" fill="currentColor"/><circle cx="15" cy="12" r="2" fill="currentColor"/><circle cx="9" cy="18" r="2" fill="currentColor"/>` + tail
	Mail         = head + `<rect x="3" y="5" width="18" height="14" rx="2"/><polyline points="3 7 12 13 21 7"/>` + tail
	Lock         = head + `<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>` + tail
	Eye          = head + `<path d="M2 12s3.5-6.5 10-6.5S22 12 22 12s-3.5 6.5-10 6.5S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>` + tail
	Folder       = head + `<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>` + tail
	File         = head + `<path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"/><polyline points="14 3 14 9 20 9"/>` + tail
	Download     = head + `<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>` + tail
	Upload       = head + `<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/>` + tail
	Refresh      = head + `<path d="M21 12a9 9 0 1 1-2.64-6.36"/><polyline points="21 3 21 9 15 9"/>` + tail
	Copy         = head + `<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>` + tail
	Calendar     = head + `<rect x="3" y="5" width="18" height="16" rx="2"/><line x1="3" y1="10" x2="21" y2="10"/><line x1="8" y1="3" x2="8" y2="7"/><line x1="16" y1="3" x2="16" y2="7"/>` + tail
	Clock        = head + `<circle cx="12" cy="12" r="9"/><polyline points="12 7 12 12 15.5 14"/>` + tail
	Play         = head + `<polygon points="6 4 20 12 6 20"/>` + tail
	Pause        = head + `<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>` + tail
	Link         = head + `<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/>` + tail
	Filter       = head + `<polygon points="22 4 2 4 10 13 10 20 14 22 14 13"/>` + tail
	Globe        = head + `<circle cx="12" cy="12" r="9"/><line x1="3" y1="12" x2="21" y2="12"/><path d="M12 3c3 3 3 15 0 18c-3-3-3-15 0-18z"/>` + tail
	Camera       = head + `<path d="M3 8a2 2 0 0 1 2-2h2l2-2h6l2 2h2a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><circle cx="12" cy="13" r="3.5"/>` + tail
	Zap          = head + `<polygon points="13 2 3 14 12 14 11 22 21 10 12 10"/>` + tail
	Message      = head + `<path d="M20 11a8 8 0 0 1-8 8H9l-5 3 1.5-5A8 8 0 1 1 20 11z"/>` + tail
)

// Named pairs a bundled icon with its name, for galleries and pickers.
type Named struct {
	Name string
	SVG  string
}

// All lists every bundled icon in alphabetical order.
var All = []Named{
	{"ArrowDown", ArrowDown}, {"ArrowLeft", ArrowLeft}, {"ArrowRight", ArrowRight}, {"ArrowUp", ArrowUp},
	{"Bell", Bell}, {"Calendar", Calendar}, {"Camera", Camera}, {"Check", Check},
	{"ChevronDown", ChevronDown}, {"ChevronLeft", ChevronLeft}, {"ChevronRight", ChevronRight}, {"ChevronUp", ChevronUp},
	{"Clock", Clock}, {"Close", Close}, {"Copy", Copy}, {"Download", Download},
	{"Edit", Edit}, {"Eye", Eye}, {"File", File}, {"Filter", Filter},
	{"Folder", Folder}, {"Globe", Globe}, {"Heart", Heart}, {"Home", Home},
	{"Info", Info}, {"Link", Link}, {"Lock", Lock}, {"Mail", Mail},
	{"Menu", Menu}, {"Message", Message}, {"Minus", Minus}, {"Moon", Moon},
	{"Pause", Pause}, {"Play", Play}, {"Plus", Plus}, {"Refresh", Refresh},
	{"Search", Search}, {"Settings", Settings}, {"Star", Star}, {"Sun", Sun},
	{"Trash", Trash}, {"Upload", Upload}, {"User", User}, {"Warning", Warning},
	{"Zap", Zap},
}
