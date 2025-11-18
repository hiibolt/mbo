import { join } from 'path';
import type { Config } from 'tailwindcss';
import forms from '@tailwindcss/forms';
import { skeleton } from '@skeletonlabs/tw-plugin';

const config = {
	darkMode: 'class',
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		join(require.resolve('@skeletonlabs/skeleton'), '../**/*.{html,js,svelte,ts}')
	],
	theme: {
		extend: {
			colors: {
				// white-inspired minimalist palette
				white: {
					darker: '#ffffff',    // Stark white background
					dark: '#f8f9fa',      // Very light grey for cards
					base: '#ffffff',       // White
					medium: '#e9ecef',     // Light grey
					light: '#dee2e6',      // Slightly darker grey
					lighter: '#adb5bd',    // Medium grey for borders
					text: '#000000',       // Black text
					'text-bright': '#000000', // Black text
					accent: '#2563eb',     // Blue for buttons
					'accent-dim': '#1d4ed8', // Darker blue on hover
					success: '#16a34a',    // Green for bid
					warning: '#ea580c',    // Orange for warning
					error: '#dc2626',      // Red for offer
				}
			}
		}
	},
	plugins: [
		forms,
		skeleton({
			themes: {
				preset: [
					{
						name: 'skeleton',
						enhancements: true
					}
				]
			}
		})
	]
} satisfies Config;

export default config;
