declare var Redoc: any;

const REDOC_URL = "https://cdn.jsdelivr.net/npm/redoc@2.0.0-rc.53/bundles/redoc.standalone.js";
const REDOC_SRI = "FAbK/5MuJ1fv6AUK+Cjrnnx8lj5Ym7TJmxv7Lli/o44Dlm5z/bF5UDQin+INTbR77xU2r5+gm7OKPG1blrBCZA==";

function percentDecode(encoded: string): string {
	let decoded = "";
	for (let i = 0; i < encoded.length; i++) {
		if (encoded[i] === '%') {
			const h = i+1<encoded.length ? parseInt(encoded[i+1], 16) : NaN;
			const l = i+2<encoded.length ? parseInt(encoded[i+2], 16) : NaN;
			if (h !== NaN && l !== NaN) {
				i += 2;
				decoded += String.fromCharCode(h * 0x10 + l);
				continue;
			}
		}
		decoded += encoded[i];
	}
	return decoded;
}

function initRedoc(percentEncodedSpec: string) {	
	const spec = JSON.parse(percentDecode(percentEncodedSpec));
	const cb = () => {
		// https://github.com/Redocly/redoc#redoc-options-object
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
		Redoc.init(spec, options, document.getElementById('redoc'));
	};
	
	const head = document.head;
	
	const l = document.createElement("link");
	l.rel = "stylesheet";
	l.href = "https://fonts.googleapis.com/css?family=Open+Sans:300,400,700|Source+Code+Pro:300,400,700&display=swap";
	head.appendChild(l);
	
	const s = document.createElement("script");
	s.src = REDOC_URL;
	s.integrity = `sha512-${REDOC_SRI}`;
	s.crossOrigin = "anonymous";
	s.onload = cb;
	head.appendChild(s);
}

export default initRedoc;
