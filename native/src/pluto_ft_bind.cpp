
#include "pluto_ft_bind.h"

STACCATO_API int bind_plutosvg_freetype(FT_Library library) {
  // Set PlutoSVG hooks for the SVG module
  if (FT_Property_Set(library, "ot-svg", "svg-hooks", &plutosvg_ft_hooks)) {
    // Handle error
    return -1;
  }

  // success
  return 0;
}

