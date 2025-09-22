// Example showing how the FilterVisitor would transform SVG elements with glow filters
// Before transformation:
const BeforeTransformation = () => (
  <svg width="200" height="200" viewBox="0 0 200 200">
    <circle id="glow-circle" cx="100" cy="100" r="50" fill="blue" />
    <rect id="glow-rect" x="25" y="25" width="150" height="150" fill="red" stroke="white" strokeWidth="2" />
  </svg>
);

// After FilterVisitor transformation (with appropriate glow filter configuration):
const AfterTransformation = () => (
  <svg width="200" height="200" viewBox="0 0 200 200">
    <defs>
      <GlowFilter 
        id="glow-circle-filter" 
        color="#00ff00" 
        intensity={2} 
        animated={true} 
        duration={2} 
        pulsing={true} 
        glowLayers={4} 
      />
      <GlowFilter 
        id="glow-rect-filter" 
        color="#ff6b6b" 
        intensity={1.5} 
        interactive={true} 
        glowLayers={3} 
      />
    </defs>
    <circle 
      id="glow-circle" 
      cx="100" 
      cy="100" 
      r="50" 
      fill="blue" 
      filter="url(#glow-circle-filter)" 
    />
    <rect 
      id="glow-rect" 
      x="25" 
      y="25" 
      width="150" 
      height="150" 
      fill="red" 
      stroke="white" 
      strokeWidth="2" 
      filter="url(#glow-rect-filter)" 
    />
  </svg>
);

export { BeforeTransformation, AfterTransformation };
