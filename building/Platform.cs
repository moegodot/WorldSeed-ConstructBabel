using System.Runtime.InteropServices;

public static class Platform
{
    public static bool Win => RuntimeInformation.IsOSPlatform(OSPlatform.Windows);
    public static bool Linux => RuntimeInformation.IsOSPlatform(OSPlatform.Linux);
    public static bool Mac => RuntimeInformation.IsOSPlatform(OSPlatform.OSX);
    
    public static bool X64 => RuntimeInformation.ProcessArchitecture == Architecture.X64;
    public static bool Arm64 => RuntimeInformation.ProcessArchitecture == Architecture.Arm64;
}
