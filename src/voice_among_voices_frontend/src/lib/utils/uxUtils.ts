const isDarkMode = () => {
  //todo: this might better be a store with an eventlistened
  return (
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
  );
};
