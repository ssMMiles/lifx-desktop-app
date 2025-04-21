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
            existingLightCard.querySelector('#label').innerText = light.label;
            existingLightCard.querySelector('#power').innerHTML = `
                <button class="power-button ${isOn ? 'on' : 'off'}" onclick="togglePower('${ip}')">
                    ${isOn ? 'On' : 'Off'}
                </button>
            `;
            continue;
        }

        const lightCard = document.createElement('div');

        lightCard.id = ip;
        lightCard.classList.add('light-card');

        lightCard.innerHTML = `
            <h2 id="label">${light.label}</h2>
            <div id="power">
                <button class="power-button ${isOn ? 'on' : 'off'}" onclick="togglePower('${ip}')">
                    ${isOn ? 'On' : 'Off'}
                </button>
            </div>
            <input type="color" id="color-picker" value="${currentColour}">
        `;

        lightCard.querySelector('#color-picker').addEventListener('input', (event) => {
            const color = event.target.value;

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