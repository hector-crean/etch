const Layout = ({ children }: { children: React.ReactNode }) => {
  return <div className="w-full h-full flex flex-col items-center justify-center">{children}</div>;
};

export default Layout;

import { motion } from 'motion/react';

const App = () => {
  return (
    <motion.div
      initial={{ opacity: 0, y: 100 }}
      exit={{ opacity: 0, y: -100 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <h1>Hello, world!</h1>
    </motion.div>
  );
};
