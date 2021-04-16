window.onload = function() {
	const REDOC_URL = "https://cdn.jsdelivr.net/npm/redoc@2.0.0-rc.53/bundles/redoc.standalone.js";
	const REDOC_SRI = "FAbK/5MuJ1fv6AUK+Cjrnnx8lj5Ym7TJmxv7Lli/o44Dlm5z/bF5UDQin+INTbR77xU2r5+gm7OKPG1blrBCZA==";
	
	const cb = function() {
		const spec = JSON.parse(decodeURI(SPEC));
		Redoc.init(spec, {
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
				menu: {
					backgroundColor: '#262a2b',
					textColor: '#fafafa'
				}
			}
		}, document.getElementById('redoc'));
	};
	
	const s = document.createElement('script');
	s.setAttribute('src', REDOC_URL);
	s.setAttribute('integrity', 'sha512-' + REDOC_SRI);
	s.setAttribute('crossorigin', 'anonymous');
	s.onload = cb;
	document.head.appendChild(s);
};
