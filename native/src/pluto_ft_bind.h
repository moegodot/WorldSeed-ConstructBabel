#pragma once

#include <plutosvg/plutosvg-ft.h>

#include <ft2build.h>

#include FT_FREETYPE_H
#include FT_MODULE_H

#include "staccato.h"

STACCATO_API int bind_plutosvg_freetype(FT_Library library);
