declare var Redoc: any;

const REDOC_URL = "https://cdn.redoc.ly/redoc/v2.1.3/bundles/redoc.standalone.js";
const REDOC_SRI = "zZxpYO8lOY5iHoiMMnOHytacuRgF/nyuy8lkiE92ajaDkgj9rtYdjNESl7PBCl04IYkRRIGs0V99lJljYBdo+g==";

function initRedoc() {
	const specElem = document.getElementById('spec');
	const redocElem = document.getElementById('redoc');
	if (specElem === null || redocElem === null) {
		console.error("Unable to find HTML elements");
		return;
	}
	
	const spec = JSON.parse(specElem.textContent ?? "");
	const bgColor = '#262a2b';
	const fgColor = '#fafafa';
	const options = {
		expandResponses: "200",
		onlyRequiredInSamples: true,
		theme: {
			typography: {
				fontFamily: '"Open Sans",sans-serif',
				fontWeightBold: '700',
				headings: {
					fontFamily: '"Open Sans",sans-serif',
				},
				code: {
					fontFamily: '"Source Code Pro",monospace'
				}
			},
			sidebar: {
				width: '20rem',
				backgroundColor: bgColor,
				textColor: fgColor
			},
			rightPanel: {
				backgroundColor: bgColor,
				textColor: fgColor
			},
			fab: {
				backgroundColor: bgColor,
				textColor: fgColor
			}
		}
	};
	Redoc.init(spec, options, redocElem);
}

function createElement(tag: string, attrs: { [key: string]: string }): HTMLElement {
	const elem = document.createElement(tag);
	for (const key in attrs) {
		elem.setAttribute(key, attrs[key]);
	}
	return elem;
}

const head = document.head;
head.appendChild(createElement("link", {
	rel: "stylesheet",
	href: "https://fonts.googleapis.com/css?family=Open+Sans:300,400,700|Source+Code+Pro:300,400,700&display=swap"
}));
const script = createElement("script", {
	src: REDOC_URL,
	integrity: `sha512-${REDOC_SRI}`,
	crossOrigin: "anonymous"
});
script.addEventListener('load', initRedoc);
head.appendChild(script);
