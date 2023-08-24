# generated from ament/cmake/core/templates/nameConfig.cmake.in

# prevent multiple inclusion
if(_nodegraph_CONFIG_INCLUDED)
  # ensure to keep the found flag the same
  if(NOT DEFINED nodegraph_FOUND)
    # explicitly set it to FALSE, otherwise CMake will set it to TRUE
    set(nodegraph_FOUND FALSE)
  elseif(NOT nodegraph_FOUND)
    # use separate condition to avoid uninitialized variable warning
    set(nodegraph_FOUND FALSE)
  endif()
  return()
endif()
set(_nodegraph_CONFIG_INCLUDED TRUE)

# output package information
if(NOT nodegraph_FIND_QUIETLY)
  message(STATUS "Found nodegraph: 0.0.1 (${nodegraph_DIR})")
endif()

# warn when using a deprecated package
if(NOT "" STREQUAL "")
  set(_msg "Package 'nodegraph' is deprecated")
  # append custom deprecation text if available
  if(NOT "" STREQUAL "TRUE")
    set(_msg "${_msg} ()")
  endif()
  # optionally quiet the deprecation message
  if(NOT ${nodegraph_DEPRECATED_QUIET})
    message(DEPRECATION "${_msg}")
  endif()
endif()

# flag package as ament-based to distinguish it after being find_package()-ed
set(nodegraph_FOUND_AMENT_PACKAGE TRUE)

# include all config extra files
set(_extras "")
foreach(_extra ${_extras})
  include("${nodegraph_DIR}/${_extra}")
endforeach()
