export function isMobile() {
  return (
    (/Android/i.test(window.navigator.userAgent)) ||
    (/iPad|iPhone|iPod/.test(navigator.userAgent) && !window.MSStream) ||
    (/Macintosh/.test(navigator.userAgent) && navigator.maxTouchPoints && navigator.maxTouchPoints > 2)
  );
}
