/**
 * URL Shortener Frontend Application
 */

document.addEventListener('DOMContentLoaded', () => {
    const form = document.getElementById('shorten-form');
    const urlInput = document.getElementById('url-input');
    const submitBtn = document.getElementById('submit-btn');
    const btnText = submitBtn.querySelector('.btn-text');
    const btnLoader = submitBtn.querySelector('.btn-loader');
    const result = document.getElementById('result');
    const shortUrl = document.getElementById('short-url');
    const originalUrl = document.getElementById('original-url');
    const copyBtn = document.getElementById('copy-btn');
    const error = document.getElementById('error');
    const errorMessage = document.getElementById('error-message');

    /**
     * Toggle loading state for the submit button
     */
    function setLoading(isLoading) {
        submitBtn.disabled = isLoading;
        btnText.hidden = isLoading;
        btnLoader.hidden = !isLoading;
    }

    /**
     * Show error message
     */
    function showError(message) {
        error.hidden = false;
        errorMessage.textContent = message;
        result.hidden = true;
    }

    /**
     * Show success result
     */
    function showResult(data) {
        error.hidden = true;
        result.hidden = false;
        shortUrl.href = data.short_url;
        shortUrl.textContent = data.short_url;
        originalUrl.textContent = truncateUrl(data.original_url, 60);
    }

    /**
     * Truncate long URLs for display
     */
    function truncateUrl(url, maxLength) {
        if (url.length <= maxLength) return url;
        return url.substring(0, maxLength) + '...';
    }

    /**
     * Copy short URL to clipboard
     */
    async function copyToClipboard(text) {
        try {
            await navigator.clipboard.writeText(text);
            copyBtn.classList.add('copied');

            // Show visual feedback
            const originalSvg = copyBtn.innerHTML;
            copyBtn.innerHTML = `
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="20 6 9 17 4 12"></polyline>
                </svg>
            `;

            setTimeout(() => {
                copyBtn.classList.remove('copied');
                copyBtn.innerHTML = originalSvg;
            }, 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
            // Fallback for older browsers
            const textArea = document.createElement('textarea');
            textArea.value = text;
            document.body.appendChild(textArea);
            textArea.select();
            document.execCommand('copy');
            document.body.removeChild(textArea);
        }
    }

    /**
     * Handle form submission
     */
    async function handleSubmit(e) {
        e.preventDefault();

        const url = urlInput.value.trim();
        if (!url) {
            showError('Please enter a URL');
            return;
        }

        // Basic URL validation
        if (!url.startsWith('http://') && !url.startsWith('https://')) {
            showError('URL must start with http:// or https://');
            return;
        }

        setLoading(true);
        error.hidden = true;
        result.hidden = true;

        try {
            const response = await fetch('/api/shorten', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ url }),
            });

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || 'Failed to shorten URL');
            }

            showResult(data);
            urlInput.value = '';
        } catch (err) {
            showError(err.message || 'An unexpected error occurred');
        } finally {
            setLoading(false);
        }
    }

    // Event listeners
    form.addEventListener('submit', handleSubmit);

    copyBtn.addEventListener('click', () => {
        const url = shortUrl.href;
        if (url && url !== '#') {
            copyToClipboard(url);
        }
    });

    // Auto-focus input on page load
    urlInput.focus();
});
