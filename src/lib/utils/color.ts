export async function extractColorTheme(imageUrl: string): Promise<{
  ambient: string;
  accent: string;
  accentDark: string;
}> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = 'Anonymous';
    img.src = imageUrl;

    img.onload = () => {
      try {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');
        if (!ctx) throw new Error('Could not get 2d context');

        canvas.width = img.width;
        canvas.height = img.height;
        ctx.drawImage(img, 0, 0);

        // Simple average color sampling (stride by 40 pixels for speed)
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
        let r = 0, g = 0, b = 0, count = 0;

        for (let i = 0; i < imageData.length; i += 4 * 40) {
          r += imageData[i];
          g += imageData[i + 1];
          b += imageData[i + 2];
          count++;
        }

        if (count > 0) {
          r = Math.round(r / count);
          g = Math.round(g / count);
          b = Math.round(b / count);
          const ambient = `rgb(${r}, ${g}, ${b})`;

          // Calculate HSL for dynamic accent
          const rNorm = r / 255;
          const gNorm = g / 255;
          const bNorm = b / 255;
          const max = Math.max(rNorm, gNorm, bNorm);
          const min = Math.min(rNorm, gNorm, bNorm);
          let h = 0, s = 0, l = (max + min) / 2;

          if (max !== min) {
            const d = max - min;
            s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
            switch (max) {
              case rNorm: h = (gNorm - bNorm) / d + (gNorm < bNorm ? 6 : 0); break;
              case gNorm: h = (bNorm - rNorm) / d + 2; break;
              case bNorm: h = (rNorm - gNorm) / d + 4; break;
            }
            h /= 6;
          }

          h = Math.round(h * 360);
          // Clamp saturation and lightness for vivid UI accents
          s = Math.max(0.55, s);
          l = Math.max(0.65, Math.min(0.75, l));
          const sPct = Math.round(s * 100);
          const lPct = Math.round(l * 100);
          const lDarkPct = Math.round(Math.max(0.40, l - 0.20) * 100); // darker variant for container

          const accent = `hsl(${h}, ${sPct}%, ${lPct}%)`;
          const accentDark = `hsl(${h}, ${sPct}%, ${lDarkPct}%)`;

          resolve({ ambient, accent, accentDark });
        } else {
          reject(new Error('No pixels sampled'));
        }
      } catch (e) {
        reject(e);
      }
    };

    img.onerror = (e) => reject(e);
  });
}
