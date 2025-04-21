function populateLights(data) {
    const container = document.getElementById('lights-container');
    const noLightsMessage = document.getElementById('no-lights-message');

    if (Object.keys(data).length === 0) {
        noLightsMessage.style.display = 'block';
        return;
    }

    noLightsMessage.style.display = 'none';

    for (let ip of Object.keys(data)) {
        const light = data[ip];
        const currentColour = hsvToHex(light.hue, light.saturation, light.brightness);
        const isOn = light.power === 65535;

        const existingLightCard = document.getElementById(ip);
        if (existingLightCard) {
            existingLightCard.querySelector('#label').innerHTML = `${light.label} <svg class="edit-icon" onclick="editLabel('${ip}')" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960" fill="#000"><path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h357l-80 80H200v560h560v-278l80-80v358q0 33-23.5 56.5T760-120H200Zm280-360ZM360-360v-170l367-367q12-12 27-18t30-6q16 0 30.5 6t26.5 18l56 57q11 12 17 26.5t6 29.5q0 15-5.5 29.5T897-728L530-360H360Zm481-424-56-56 56 56ZM440-440h56l232-232-28-28-29-28-231 231v57Zm260-260-29-28 29 28 28 28-28-28Z"/></svg>`;
            existingLightCard.querySelector('#power').innerHTML = `
                <button class="power-button ${isOn ? 'on' : 'off'}" onclick="togglePower('${ip}')">
                    <svg version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
	 viewBox="0 0 30.143 30.143" xml:space="preserve">
<g>
	<path style="fill:#030104;" d="M20.034,2.357v3.824c3.482,1.798,5.869,5.427,5.869,9.619c0,5.98-4.848,10.83-10.828,10.83
		c-5.982,0-10.832-4.85-10.832-10.83c0-3.844,2.012-7.215,5.029-9.136V2.689C4.245,4.918,0.731,9.945,0.731,15.801
		c0,7.921,6.42,14.342,14.34,14.342c7.924,0,14.342-6.421,14.342-14.342C29.412,9.624,25.501,4.379,20.034,2.357z"/>
	<path style="fill:#030104;" d="M14.795,17.652c1.576,0,1.736-0.931,1.736-2.076V2.08c0-1.148-0.16-2.08-1.736-2.08
		c-1.57,0-1.732,0.932-1.732,2.08v13.496C13.062,16.722,13.225,17.652,14.795,17.652z"/>
</g>
</svg>
                </button>
            `;
            continue;
        }

        const lightCard = document.createElement('div');

        lightCard.id = ip;
        lightCard.classList.add('light-card');

        lightCard.innerHTML = `
            <h2 id="label">${light.label} <svg class="edit-icon" onclick="editLabel('${ip}')" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960" fill="#000"><path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h357l-80 80H200v560h560v-278l80-80v358q0 33-23.5 56.5T760-120H200Zm280-360ZM360-360v-170l367-367q12-12 27-18t30-6q16 0 30.5 6t26.5 18l56 57q11 12 17 26.5t6 29.5q0 15-5.5 29.5T897-728L530-360H360Zm481-424-56-56 56 56ZM440-440h56l232-232-28-28-29-28-231 231v57Zm260-260-29-28 29 28 28 28-28-28Z"/></svg></h2>
            <div class="controls">
                <div id="power">
                    <button class="power-button ${isOn ? 'on' : 'off'}" onclick="togglePower('${ip}')">
                        <svg version="1.1" id="Capa_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" 
	 viewBox="0 0 30.143 30.143" xml:space="preserve">
<g>
	<path style="fill:#030104;" d="M20.034,2.357v3.824c3.482,1.798,5.869,5.427,5.869,9.619c0,5.98-4.848,10.83-10.828,10.83
		c-5.982,0-10.832-4.85-10.832-10.83c0-3.844,2.012-7.215,5.029-9.136V2.689C4.245,4.918,0.731,9.945,0.731,15.801
		c0,7.921,6.42,14.342,14.34,14.342c7.924,0,14.342-6.421,14.342-14.342C29.412,9.624,25.501,4.379,20.034,2.357z"/>
	<path style="fill:#030104;" d="M14.795,17.652c1.576,0,1.736-0.931,1.736-2.076V2.08c0-1.148-0.16-2.08-1.736-2.08
		c-1.57,0-1.732,0.932-1.732,2.08v13.496C13.062,16.722,13.225,17.652,14.795,17.652z"/>
</g>
</svg>
                    </button>
                </div>
                <hex-color-picker id="color-picker" color="${currentColour}"></hex-color-picker>
            </div>
        `;

        lightCard.querySelector('#color-picker').addEventListener('color-changed', (event) => {
            const color = event.detail.value;

            const [r, g, b] = hexToRgb(color);
            const { h, s, v } = RGBtoHSV(r, g, b);
            const kelvin = light.kelvin; 

            console.log(`HSL: ${h}, ${s}, ${v}`);

            fetch(`/api/setColor?ip=${ip}&hue=${Math.floor(h * 65535)}&saturation=${Math.floor(s * 65535)}&brightness=${Math.floor(v * 65535)}&kelvin=${kelvin}`, {
                method: 'POST',
            })
                .then(response => response.json())
                .then(data => console.log(data))
                .catch(error => console.error('Error:', error));
        });

        container.appendChild(lightCard);
    }
}

async function togglePower(lightIp) {
    try {
        const response = await fetch(`/api/setPower?ip=${lightIp}`, {
            method: 'POST',
        });

        if (response.ok) {
            console.log(`Power toggled for ${lightIp}`);
        } else {
            console.error('Failed to toggle power:', response.statusText);
        }
    } catch (error) {
        console.error('Error toggling power:', error);
    }
}

async function refreshLightData() {
    try {
        // disable cors
        const response = await fetch('/api/lights', {
            // mode: 'no-cors'
            headers: {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
            }
        });

        const data = await response.json();

        populateLights(data);
    } catch (error) {
        console.error('Error fetching light data:', error);
    }
}

function editLabel(ip) {
    const lightCard = document.getElementById(ip);
    const labelElement = lightCard.querySelector('#label');
    const currentLabel = labelElement.innerText.replace(' Edit', '');
    const input = document.createElement('input');
    input.type = 'text';
    input.value = currentLabel;
    input.classList.add('edit-label-input');
    labelElement.replaceWith(input);
    input.focus();
    input.addEventListener('keypress', function(event) {
        if (event.key === 'Enter') {
            const newLabel = input.value;
            fetch('/api/setName', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ ip: ip, name: newLabel })
            })
            .then(() => {
                const newLabelEl = document.createElement('h2');
                newLabelEl.id = 'label';
                newLabelEl.innerHTML = `${newLabel} <svg class="edit-icon" onclick="editLabel('${ip}')" width="16" height="16" xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960" fill="#000"><path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h357l-80 80H200v560h560v-278l80-80v358q0 33-23.5 56.5T760-120H200Zm280-360ZM360-360v-170l367-367q12-12 27-18t30-6q16 0 30.5 6t26.5 18l56 57q11 12 17 26.5t6 29.5q0 15-5.5 29.5T897-728L530-360H360Zm481-424-56-56 56 56ZM440-440h56l232-232-28-28-29-28-231 231v57Zm260-260-29-28 29 28 28 28-28-28Z"/></svg>`;
                input.replaceWith(newLabelEl);
            })
            .catch(error => console.error('Error:', error));
        }
    });
}

setInterval(refreshLightData, 500);

function componentToHex(c) {
    var hex = c.toString(16);
    return hex.length == 1 ? "0" + hex : hex;
}

function rgbToHex(r, g, b) {
    return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
}

function hexToRgb(hex) {
    const bigint = parseInt(hex.slice(1), 16);
    const r = (bigint >> 16) & 255;
    const g = (bigint >> 8) & 255;
    const b = bigint & 255;
    return [r, g, b];
}

function RGBtoHSV(r, g, b) {
    if (arguments.length === 1) {
        g = r.g, b = r.b, r = r.r;
    }
    var max = Math.max(r, g, b), min = Math.min(r, g, b),
        d = max - min,
        h,
        s = (max === 0 ? 0 : d / max),
        v = max / 255;

    switch (max) {
        case min: h = 0; break;
        case r: h = (g - b) + d * (g < b ? 6 : 0); h /= 6 * d; break;
        case g: h = (b - r) + d * 2; h /= 6 * d; break;
        case b: h = (r - g) + d * 4; h /= 6 * d; break;
    }

    return {
        h: h,
        s: s,
        v: v
    };
}

// ai
function HSVtoRGB(h, s, v) {
    var r, g, b, i, f, p, q, t;
    if (arguments.length === 1) {
        s = h.s, v = h.v, h = h.h;
    }
    i = Math.floor(h * 6);
    f = h * 6 - i;
    p = v * (1 - s);
    q = v * (1 - f * s);
    t = v * (1 - (1 - f) * s);
    switch (i % 6) {
        case 0: r = v, g = t, b = p; break;
        case 1: r = q, g = v, b = p; break;
        case 2: r = p, g = v, b = t; break;
        case 3: r = p, g = q, b = v; break;
        case 4: r = t, g = p, b = v; break;
        case 5: r = v, g = p, b = q; break;
    }
    return {
        r: r * 255,
        g: g * 255,
        b: b * 255
    };
}

function hsvToHex(h, s, v) {
    // Normalize HSV values
    h = (h - 1) / 65564 * 360;  // Normalize hue to [0, 360]
    s = (s - 1) / 65564;         // Normalize saturation to [0, 1]
    v = (v - 1) / 65564;         // Normalize value to [0, 1]
  
    // Convert HSV to RGB
    let c = v * s;
    let x = c * (1 - Math.abs((h / 60) % 2 - 1));
    let m = v - c;
  
    let r, g, b;
    if (0 <= h && h < 60) {
      r = c; g = x; b = 0;
    } else if (60 <= h && h < 120) {
      r = x; g = c; b = 0;
    } else if (120 <= h && h < 180) {
      r = 0; g = c; b = x;
    } else if (180 <= h && h < 240) {
      r = 0; g = x; b = c;
    } else if (240 <= h && h < 300) {
      r = x; g = 0; b = c;
    } else {
      r = c; g = 0; b = x;
    }
  
    r = Math.round((r + m) * 255);
    g = Math.round((g + m) * 255);
    b = Math.round((b + m) * 255);
  
    // Convert RGB to hex
    return "#" + ((1 << 24) + (r << 16) + (g << 8) + b).toString(16).slice(1).toUpperCase();
}