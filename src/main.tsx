import React from 'react';
import ReactDOM from 'react-dom/client';
import './style.css';
import { Sprite } from './Sprite';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><main aria-label="Crawde desktop companion"><Sprite animation="idle" /></main></React.StrictMode>,
);
