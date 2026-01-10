using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using JetBrains.Annotations;
using Nuke.Common;
using Nuke.Common.IO;
using Nuke.Common.Tooling;
using Nuke.Common.Tools.DotNet;
using Nuke.Common.Tools.Git;
using Nuke.Common.Utilities.Collections;
using Serilog;

class Build : NukeBuild
{
    /// Support plugins are available for:
    ///   - JetBrains ReSharper        https://nuke.build/resharper
    ///   - JetBrains Rider            https://nuke.build/rider
    ///   - Microsoft Visual Studio    https://nuke.build/visualstudio
    ///   - Microsoft VSCode           https://nuke.build/vscode
    public static int Main() => Execute<Build>(x => x.BuildAll);

    public static bool IsStandardAlongProject()
    {
        return string.IsNullOrWhiteSpace(GitTasks.Git("rev-parse --show-superproject-working-tree")
            .Aggregate(string.Empty, (pre, cur) => $"{pre}{cur}"));
    }

    public static AbsolutePath ExternalLibraryPath => RootDirectory / "library";
    
    public static AbsolutePath NativePath => RootDirectory / "native";
    
    [Parameter("Configuration to build - Default is 'Debug' (standalone project) or 'Release' (as a git submodule)")]
    readonly Configuration Configuration = IsStandardAlongProject() ? Configuration.Debug : Configuration.Release;

    [Parameter("Should build samples - Default is 'true' (standalone project) or 'false' (as a git submodule)")]
    readonly bool ShouldBuildSample = IsStandardAlongProject();

    [Parameter("Use cmake from this path.")]
    readonly string CMake = "cmake";
    
    [Parameter("Use clang from this path." +
               "Note: this will find more tools like clang++,lld etc. in the directory containing the clang.")]
    readonly string Clang = "clang";
    
    [Parameter("Use uv from this path.")]
    readonly string Uv = "uv";
    
    [Parameter("Prefer using tool that contains this string in its path when multiple tools are found in PATH.")]
    readonly string PreferTool = "homebrew";
    
    AbsolutePath ExternalLibraryBuildPath => RootDirectory / $"build-{Configuration}";
    
    AbsolutePath NativeBuildPath => ExternalLibraryBuildPath / "native";
    
    AbsolutePath ExternalLibraryInstallPath => RootDirectory / $"install-{Configuration}";
    
    AbsolutePath ArtifactPath => RootDirectory / $"artifact-{Configuration}";
    
    AbsolutePath NativeInstallPath => ArtifactPath / "native";
    
    AbsolutePath ScriptPath => RootDirectory / "script";
    
    AbsolutePath RustSourcePath => RootDirectory / "src";
    
    AbsolutePath CargoFilePath => RustSourcePath / "Cargo.toml";

    AbsolutePath VersionFile => RootDirectory / "version.txt";
    
    Target RestoreGitSubmodules => _ => _.Executes(() =>
    {
        GitTasks.Git("submodule update --init --recursive");
    });
    
    Target RestoreNative => _ => _.DependsOn(RestoreGitSubmodules);

    private static List<string> Which(string name, string preferred = "homebrew")
    {
        if (Path.IsPathRooted(name))
        {
            return [name];
        }
        // find from PATH
        var path = Environment.GetEnvironmentVariable("PATH");

        if (path == null)
        {
            return [name];
        }

        List<string> suffixes = [string.Empty];
        if (Platform.Win)
        {
            // read `PATHEXT` environment variable
            var pathext = Environment.GetEnvironmentVariable("PATHEXT");
            if (!string.IsNullOrEmpty(pathext))
            {
                suffixes.AddRange(pathext.Split(';'));
                suffixes.AddRange(pathext.Split(';').Select(s => s.ToLowerInvariant()));
            }
        }

        var paths = path.Split(Path.PathSeparator);
        List<string> results = [];

        foreach (var p in paths)
        {
            foreach (var suffix in suffixes)
            {
                var fullPath = Path.Combine(p, name + suffix);
                if (File.Exists(fullPath))
                {
                    results.Add(fullPath);
                }
            }
        }
        
        // find homebrew version
        List<string> prefers = [];
        List<string> others = [];
        
        if (preferred != string.Empty)
        {
            foreach (var result in results)
            {
                if (result.Contains(preferred))
                {
                    prefers.Add(result);
                }
                else
                {
                    others.Add(result);
                }
            }
        }
        else
        {
            others = results;
        }

        return [..prefers,..others];
    }

    private static void RunTool(string name, string pwd,string[] args,[CanBeNull] Action<IDictionary<string,string?>> env = null)
    {
        var info = new ProcessStartInfo
        {
            FileName = string.IsNullOrEmpty(Path.GetExtension(name)) && Platform.Win ? $"{name}.exe" : name,
            CreateNoWindow = true,
            RedirectStandardError = false,
            RedirectStandardInput = false,
            RedirectStandardOutput = false,
            UseShellExecute = false,
            WorkingDirectory = pwd,
        };
        foreach (var arg in args)
        {
            info.ArgumentList.Add(arg);
        }
        
        env?.Invoke(info.Environment);

        var proc = Process.Start(info);
        if (proc == null)
        {
            throw new InvalidOperationException($"failed to start `${name}` process");
        }
        proc.WaitForExit();
        if (proc.ExitCode != 0)
        {
            throw new InvalidOperationException($"`{name}` process exit with non-zero code {proc}");
        }
    }

    private void RunUv(string pwd, params string[] args)
    {
        var uv = Which(Uv,PreferTool).First();
        
        var clang = Which(Clang,PreferTool).First();
        var clangPlusPlus = Which(string.IsNullOrEmpty(Path.GetDirectoryName(clang)) ? "clang++" : $"{Path.GetDirectoryName(clang)}/clang++",PreferTool).First();
        var ar = Which(string.IsNullOrEmpty(Path.GetDirectoryName(clang)) ? "llvm-ar" : $"{Path.GetDirectoryName(clang)}/llvm-ar",PreferTool).First();
        var ranlib =
            Which(
                string.IsNullOrEmpty(Path.GetDirectoryName(clang))
                    ? "llvm-ranlib"
                    : $"{Path.GetDirectoryName(clang)}/llvm-ranlib", PreferTool).First();
        
        Log.Information("Using uv: {Uv}", uv);
        Log.Information("Using clang: {Clang}", clang);
        Log.Information("Using clang++: {ClangPlusPlus}", clangPlusPlus);
        Log.Information("Using ar: {Ar}", ar);
        Log.Information("Using ranlib: {Ranlib}", ranlib);
        
        RunTool(uv ?? "uv", pwd, args, dictionary =>
        {
            dictionary["CC"] = clang;
            dictionary["CXX"] = clangPlusPlus;
            dictionary["AR"] = ar;
            dictionary["RANLIB"] = ranlib;
        });
    }
    
    private void RunCMake(string pwd, params string[] args)
    {
        var cmake = Which(CMake,PreferTool).First();
        Log.Information("Run {CMake} \"{Arguments}\"", cmake, string.Join("\" \"", args));
        RunTool(cmake ?? "cmake", pwd, args);
    }
    
    private AbsolutePath GetSourcePathOf(string lib)
    {
        if(lib == "native")
        {
            return NativePath;
        }
        return ExternalLibraryPath / lib;
    }

    private AbsolutePath GetBuildPath(string lib)
    {
        if (lib == "native")
        {
            return NativeBuildPath;
        }
        return ExternalLibraryBuildPath / lib;
    }
    
    private AbsolutePath GetInstallPathOf(string lib)
    {
        if (lib == "native")
        {
            return NativeInstallPath;
        }

        return ExternalLibraryInstallPath;
    }

    private AbsolutePath GetInstallRoot()
    {
        return ExternalLibraryInstallPath;
    }

    private bool HitCache(string lib)
    {
        var install = GetInstallPathOf(lib);
        var lockFile = install / $"{lib}-installed.lock";
        if (!File.Exists(lockFile)) return false;
        Log.Information("Hit cache lock file {Cache}", lockFile);
        return true;
    }

    private bool BuildCMake(string lib, string[] options)
    {
        var src = GetSourcePathOf(lib);
        var build = GetBuildPath(lib);
        var install = GetInstallPathOf(lib);
        var lockFile = install / $"{lib}-installed.lock";

        try
        {
            if (File.Exists(lockFile))
            {
                Log.Information("Hit catch {Cache}", install);
                return false;
            }
        }
        catch (Exception)
        {
            // as lock do not exists
        }

        RunCMake(src, ["-S", src, "-B", build, "-G", "Ninja",
            $"-DCMAKE_TOOLCHAIN_FILE={NativePath / "toolchain.cmake"}",
            $"-DCMAKE_BUILD_TYPE={Configuration}",
            $"-DCMAKE_INSTALL_PREFIX={install}",
            $"-DSTACCATO_INSTALL_ROOT={GetInstallRoot()}",
            "-Wno-dev",
            ..options]);
        RunCMake(build,["--build", ".", "--config",Configuration]);
        RunCMake(build,["--install", ".", "--config",Configuration]);

        File.Create(lockFile).Close();

        return true;
    }
    
    private void BuildMeson(string lib, params string[] options)
    {
        var src = GetSourcePathOf(lib);
        var build = GetBuildPath(lib);
        var install = GetInstallPathOf(lib);
        var lockFile = install / $"{lib}-installed.lock";

        try
        {
            if (File.Exists(lockFile))
            {
                Log.Information("Hit catch {Cache}", install);
                return;
            }
        }
        catch (Exception)
        {
            // as lock do not exists
        }

        RunUv(src, ["run","--project",ScriptPath, "meson" ,"setup", build, "--prefix", install, "--buildtype", Configuration.ToString().ToLowerInvariant(),..options]);
        RunUv(build,["run","--project",ScriptPath, "meson", "compile"]);
        RunUv(build,["run","--project",ScriptPath, "meson", "install"]);

        File.Create(lockFile).Close();
    }
    
    Target BuildHarfbuzz => _ => _.DependsOn(RestoreNative)
        .OnlyWhenDynamic(() => !HitCache("harfbuzz"))
        .Executes(() =>
        {
            BuildMeson("harfbuzz",
                "-D","backend=ninja",
                "-D","b_lto=true",
                "-D", "b_lto_mode=thin",
                "-D","default_library=static",
                "-D", "auto_features=disabled",
                "-D", "icu=disabled",
                "-D", "cairo=disabled",
                "-D", "chafa=disabled",
                "-D", "freetype=disabled",
                "-D", "glib=disabled",
                "-D", "gobject=disabled",
                "-D", "icu=disabled",
                "-D", "tests=disabled",
                "-D", "utilities=disabled",
                "-D", "b_pie=true",
                "-D", "b_staticpic=true"
            );
        });
    
    Target BuildPlutosvg => _ => _.DependsOn(RestoreNative)
        .DependsOn(BuildFreeType)
        .OnlyWhenDynamic(() => !HitCache("plutosvg"))
        .Executes(() =>
        {
            BuildCMake("plutosvg",[
                "-DPLUTOSVG_BUILD_EXAMPLES=OFF",
                "-DPLUTOSVG_ENABLE_FREETYPE=ON"]
            );
        });
    
    Target BuildZLib => _ => _.DependsOn(RestoreNative)
        .OnlyWhenDynamic(() => !HitCache("zlib"))
        .Executes(() =>
        {
            BuildCMake("zlib", [
                "-DZLIB_ENABLE_TESTS=OFF",
                "-DZLIB_COMPAT=ON",
                "-DWITH_GTEST=OFF",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DBUILD_STATIC_LIBS=ON",
                "-DWITH_ARMV6=OFF",
                "-DWITH_NATIVE_INSTRUCTIONS=OFF"]
            );
        });
    
    Target BuildBZip2 => _ => _.DependsOn(RestoreNative)
        .OnlyWhenDynamic(() => !HitCache("bzip2"))
        .Executes(() =>
        {
            var suffix = Platform.Win ? "lib" : "a";
            
            BuildCMake("bzip2", [
                "-DENABLE_WERROR=OFF",
                "-DENABLE_APP=OFF",
                "-DENABLE_DEBUG=OFF",
                "-DENABLE_DOCS=OFF",
                "-DENABLE_EXAMPLES=OFF",
                "-DENABLE_LIB_ONLY=ON",
                "-DENABLE_SHARED_LIB=OFF",
                "-DENABLE_STATIC_LIB=ON"]
            );

            if (!File.Exists(GetInstallPathOf("bzip2") / "lib" / $"libbz2.{suffix}"))
            {
                File.CreateSymbolicLink(
                    GetInstallPathOf("bzip2") / "lib" / $"libbz2.{suffix}",
                    GetInstallPathOf("bzip2") / "lib" / $"libbz2_static.{suffix}"
                );
            }
        });
    
    Target BuildLibPng => _ => _.DependsOn(RestoreNative)
        .DependsOn(BuildZLib)
        .OnlyWhenDynamic(() => !HitCache("libpng"))
        .Executes(() =>
        {
            BuildCMake("libpng",[
                "-DPNG_TESTS=OFF",
                "-DPNG_EXECUTABLES=OFF",
                "-DPNG_BUILD_ZLIB=OFF",
                "-DPNG_HARDWARE_OPTIMIZATIONS=ON",
                "-DPNG_TOOLS=OFF",
                $"-DZLIB_ROOT={GetInstallPathOf("zlib")}",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DBUILD_STATIC_LIBS=ON",
                "-DPNG_SHARED=OFF",
                "-DPNG_STATIC=ON"]
            );
        });
    
    Target BuildBrotli => _ => _.DependsOn(RestoreNative)
        .OnlyWhenDynamic(() => !HitCache("brotli"))
        .Executes(() =>
        {
            BuildCMake("brotli",[
                "-DBROTLI_BUILD_TOOLS=OFF",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DBUILD_STATIC_LIBS=ON"]
            );
        });
    
    Target BuildFreeType => _ => _.DependsOn(RestoreNative)
        .DependsOn(BuildLibPng)
        .DependsOn(BuildZLib)
        .DependsOn(BuildBZip2)
        .DependsOn(BuildBrotli)
        .OnlyWhenDynamic(() => !HitCache("freetype"))
        .Executes(() =>
        {
            var suffix = Platform.Win ? "lib" : "a";
            
            BuildCMake("freetype",[
                "-DFT_REQUIRE_ZLIB=ON",
                "-DFT_REQUIRE_BZIP2=ON",
                "-DFT_REQUIRE_PNG=ON",
                "-DFT_REQUIRE_BROTLI=ON",
                "-DFT_REQUIRE_HARFBUZZ=ON",
                "-DFT_DYNAMIC_HARFBUZZ=ON",
                $"-DZLIB_ROOT={GetInstallPathOf("zlib")}",
                $"-DBROTLIDEC_ROOT={GetInstallPathOf("brotli")}",
                $"-DPNG_ROOT={GetInstallPathOf("libpng")}",
                $"-DBZIP2_ROOT={GetInstallPathOf("bzip2")}",
                $"-DBZIP2_INCLUDE_DIRS={GetInstallPathOf("bzip2")/"include"}",
                $"-DBZIP2_LIBRARIES={GetInstallPathOf("bzip2") / "lib" / $"libbz2_static.{suffix}"}",
                $"-DBZIP2_LIBRARY_DEBUG={GetInstallPathOf("bzip2") / "lib" / $"libbz2_static.{suffix}"}",
                $"-DBZIP2_LIBRARY_RELEASE={GetInstallPathOf("bzip2") / "lib" / $"libbz2_static.{suffix}"}",
                "-DBZIP2_NEED_PREFIX=ON",
                "-DBUILD_SHARED_LIBS=OFF",
                "-DBUILD_STATIC_LIBS=ON"]
            );
        });
    
    Target BuildSdl => _ => _.DependsOn(RestoreNative)
        .OnlyWhenDynamic(() => !HitCache("SDL"))
        .Executes(() =>
        {
            string[] libusb = Platform.Linux ? ["-DSDL_HIDAPI_LIBUSB=ON", "-DSDL_HIDAPI_LIBUSB_SHARED=ON"] :
            ["-DSDL_HIDAPI_LIBUSB=OFF", "-DSDL_HIDAPI_LIBUSB_SHARED=OFF"];
            BuildCMake("SDL",
                ["-DSDL_INSTALL=ON", "-DSDL_DEPS_SHARED=ON",
                    "-DSDL_SHARED=ON", "-DSDL_STATIC=OFF",
                    "-DSDL_TESTS=OFF", "-DSDL_UNINSTALL=OFF",
                    "-DSDL_EXAMPLES=OFF",
                ..libusb]
            );
        });

    Target BuildNative => _ => _.DependsOn(RestoreNative)
        .DependsOn(BuildFreeType)
        .DependsOn(BuildSdl)
        .DependsOn(BuildHarfbuzz)
        .DependsOn(BuildPlutosvg)
        .OnlyWhenDynamic(() => !HitCache("native"))
        .Executes(() =>
        {
            BuildCMake("native",[]);
            
            // copy SDL3
            // this should be the only dynamic library we used
            // other dynamic libraries are system libraries
            var sdlLibName = Platform.Win ? "SDL3.dll" : Platform.Mac ? "libSDL3.dylib" : "libSDL3.so";
            if (!File.Exists(NativeInstallPath / sdlLibName))
            {
                File.CreateSymbolicLink(NativeInstallPath / sdlLibName,
                    GetInstallPathOf("SDL") / "lib" / sdlLibName);
            }
        });

    Target Clean => _ => _
        .Executes(() =>
        {
            if (Directory.Exists(ExternalLibraryBuildPath))
            {
                Directory.Delete(ExternalLibraryBuildPath, true);
            }

            if (Directory.Exists(NativeBuildPath))
            {
                Directory.Delete(NativeBuildPath, true);
            }
        });
    
    Target BuildRust => _ => _
        .DependsOn(UpdateVersionFiles)
        .DependsOn(BuildNative)
        .Executes(() =>
        {
            string profile;

            if (Configuration == Configuration.Release)
            {
                profile = "release";
            }
            else if(Configuration == Configuration.Debug)
            {
                profile = "dev";
            }
            else
            {
                profile = Configuration.ToString().ToLowerInvariant();
            }
            
            RunTool("cargo", RustSourcePath, ["build", "--profile", profile, "--workspace", "-Z", "build-std=core,alloc,std,proc_macro,test"]);
        });
    
    Target BuildSample => _ => _
        .DependsOn(RestoreNative)
        .OnlyWhenDynamic(() => ShouldBuildSample)
        .Executes(() =>
        {
            
        });

    Target UpdateVersionFiles => _ => _
        .Executes(() =>
        {
            var cargo = File.ReadAllText(CargoFilePath);
            var version = File.ReadAllText(VersionFile).Trim();
            
            Log.Information("Use {Version}", version);

            var startMark = "# THIS IS UPDATED BY BUILD SCRIPT - DO NOT MODIFY MANUALLY - START";
            var endMark = "# THIS IS UPDATED BY BUILD SCRIPT - DO NOT MODIFY MANUALLY - END";
            
            var start =  cargo.IndexOf(startMark,StringComparison.Ordinal);
            var end = cargo.IndexOf(endMark, StringComparison.Ordinal);

            var result = cargo.Substring(0, start + startMark.Length) + "\n" + $"version = \"{version}\"" + "\n" + cargo.Substring(end);
            
            Log.Information("Update {File}", CargoFilePath);
            
            File.WriteAllText(CargoFilePath,result);
        });

    Target BuildAll => _ => _.DependsOn(BuildNative).DependsOn(BuildSample).DependsOn(BuildRust);
}
