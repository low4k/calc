import { createApp } from './app/App.ts';
import 'mathlive/fonts.css';
import './styles.css';

const target = document.querySelector<HTMLDivElement>('#app');

if (!target) {
  throw new Error('Missing app root');
}

target.appendChild(createApp());
