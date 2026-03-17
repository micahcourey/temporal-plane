import Header from './components/Header';
import Hero from './components/Hero';
import Features from './components/Features';
import DetailedFeatures from './components/DetailedFeatures';
import HowItWorks from './components/HowItWorks';
import Footer from './components/Footer';
import './App.css';

function App() {
  return (
    <div className="app-container">
      <Header />
      <main>
        <Hero />
        <Features />
        <DetailedFeatures />
        <HowItWorks />
      </main>
      <Footer />
    </div>
  );
}

export default App;
