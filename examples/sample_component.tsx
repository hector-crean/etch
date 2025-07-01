import React from 'react';
import logoImage from './assets/images/logo.png';
import heroBackground from '../shared/images/hero-bg.jpg';
import { IconType } from './types';

// Example component showing different ways assets can be referenced
export const SampleComponent: React.FC = () => {
  // String literal asset reference
  const iconPath = './icons/search.svg';
  
  // Asset in template literal
  const backgroundStyle = {
    backgroundImage: `url('./assets/patterns/texture.png')`
  };

  // Array of asset paths
  const galleryImages = [
    './gallery/image1.jpg',
    './gallery/image2.jpg',
    './gallery/image3.png'
  ];

  return (
    <div className="sample-component">
      {/* Direct import usage */}
      <img src={logoImage} alt="Company Logo" className="logo" />
      
      {/* Background image from import */}
      <div 
        className="hero-section"
        style={{ backgroundImage: `url(${heroBackground})` }}
      >
        <h1>Welcome</h1>
      </div>

      {/* JSX attribute with relative path */}
      <img src="./assets/icons/user.png" alt="User Icon" className="user-icon" />
      
      {/* Icon from string variable */}
      <img src={iconPath} alt="Search" className="search-icon" />
      
      {/* Video asset */}
      <video controls>
        <source src="./media/demo.mp4" type="video/mp4" />
        <source src="./media/demo.webm" type="video/webm" />
      </video>
      
      {/* Audio asset */}
      <audio controls>
        <source src="./audio/notification.mp3" type="audio/mpeg" />
        <source src="./audio/notification.wav" type="audio/wav" />
      </audio>
      
      {/* Document link */}
      <a href="./documents/manual.pdf" download>Download Manual</a>
      
      {/* Background with template literal */}
      <div style={backgroundStyle} className="textured-section">
        <p>This section has a textured background</p>
      </div>
      
      {/* Gallery of images */}
      <div className="gallery">
        {galleryImages.map((imagePath, index) => (
          <img 
            key={index}
            src={imagePath} 
            alt={`Gallery item ${index + 1}`}
            className="gallery-item"
          />
        ))}
      </div>
      
      {/* Complex asset usage in data attributes */}
      <div 
        data-bg-image="./assets/backgrounds/pattern.svg"
        data-icon="./assets/icons/star.svg"
        className="decorated-section"
      >
        <span>Decorated content</span>
      </div>
    </div>
  );
};

export default SampleComponent; 