declare var Redoc: any;

const REDOC_URL = "https://cdn.jsdelivr.net/npm/redoc@2.0.0-rc.59/bundles/redoc.standalone.js";
const REDOC_SRI = "VCgPVm38H6H9atNzhllxdvvQCi0regBhB0pahcKnaSBgkzS75Od3wh9ljB6aFVoWCTZ9I36Hk0EgBXbPKSRynw==";

function initRedoc() {
	const specElem = document.getElementById('spec');
	const redocElem = document.getElementById('redoc');
	if (specElem === null || redocElem === null) {
		console.error("Unable to find HTML elements");
		return;
	}
	
	const spec = JSON.parse(specElem.textContent ?? "");
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
				backgroundColor: '#262a2b',
				textColor: '#fafafa'
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
