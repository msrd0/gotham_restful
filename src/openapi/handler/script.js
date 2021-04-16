window.onload = function() {
	const REDOC_URL = "https://cdn.jsdelivr.net/npm/redoc@2.0.0-rc.40/bundles/redoc.standalone.js";
	const REDOC_SRI = "iGByzgA/G8tZoUo2VEs3tLMgXADp7QDFhIz/JbpdnF5Iy0ehv01g2mTqehbRs54g/C2pMKaSNDHV+gpur+aErA==";
	
	const cb = function() {
		Redoc.init(window.location.origin + window.location.pathname + '?spec', {
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
