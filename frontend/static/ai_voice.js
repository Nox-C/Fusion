// AI Orb Voice Command & Speech Synthesis Handler
// Uses Web Speech API for speech recognition and synthesis

export class AIVoice {
    constructor(onTranscript, onListening, onSpeaking) {
        this.recognition = null;
        this.synth = window.speechSynthesis;
        this.onTranscript = onTranscript;
        this.onListening = onListening;
        this.onSpeaking = onSpeaking;
        this.isListening = false;
        this.isSpeaking = false;
        this.initRecognition();
    }

    initRecognition() {
        const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
        if (!SpeechRecognition) return;
        this.recognition = new SpeechRecognition();
        this.recognition.lang = 'en-US';
        this.recognition.interimResults = false;
        this.recognition.continuous = false;

        this.recognition.onstart = () => {
            this.isListening = true;
            if (this.onListening) this.onListening(true);
        };
        this.recognition.onend = () => {
            this.isListening = false;
            if (this.onListening) this.onListening(false);
        };
        this.recognition.onresult = (event) => {
            const transcript = Array.from(event.results)
                .map(r => r[0].transcript)
                .join('');
            if (this.onTranscript) this.onTranscript(transcript);
        };
        this.recognition.onerror = (event) => {
            this.isListening = false;
            if (this.onListening) this.onListening(false);
        };
    }

    startListening() {
        if (this.recognition && !this.isListening) {
            this.recognition.start();
        }
    }

    stopListening() {
        if (this.recognition && this.isListening) {
            this.recognition.stop();
        }
    }

    speak(text) {
        if (!this.synth) return;
        this.isSpeaking = true;
        if (this.onSpeaking) this.onSpeaking(true);
        const utter = new SpeechSynthesisUtterance(text);
        utter.lang = 'en-US';
        utter.rate = 1.05;
        utter.pitch = 1.12;
        utter.onend = () => {
            this.isSpeaking = false;
            if (this.onSpeaking) this.onSpeaking(false);
        };
        this.synth.speak(utter);
    }
}

// Singleton for dashboard
window.FusionAIVoice = new AIVoice();
